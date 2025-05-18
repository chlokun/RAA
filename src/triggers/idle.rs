use anyhow::Result;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use crate::webhook::{EventCategory, WebhookSender};

pub struct IdleMonitor {
    webhook: WebhookSender,
    idle_threshold: Duration,
    check_interval: Duration,
    last_activity: Arc<Mutex<Instant>>,
    running: Arc<Mutex<bool>>,
}

impl IdleMonitor {
    pub fn new(webhook: WebhookSender, idle_minutes: u64) -> Self {
        Self {
            webhook,
            idle_threshold: Duration::from_secs(idle_minutes * 60),
            check_interval: Duration::from_secs(60), // Check every minute
            last_activity: Arc::new(Mutex::new(Instant::now())),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn start_monitoring(&self) -> Result<()> {
        let last_activity = Arc::clone(&self.last_activity);
        let running = Arc::clone(&self.running);
        let webhook = self.webhook.clone();
        let idle_threshold = self.idle_threshold;
        let check_interval = self.check_interval;
        
        // Set running to true
        *running.lock().unwrap() = true;
        
        thread::spawn(move || {
            let mut was_idle = false;
            
            while *running.lock().unwrap() {
                // Sleep for the check interval
                thread::sleep(check_interval);
                
                // Check idle time
                let idle_time = {
                    let last = last_activity.lock().unwrap();
                    last.elapsed()
                };
                
                // If we've crossed the idle threshold and weren't previously idle
                if idle_time >= idle_threshold && !was_idle {
                    was_idle = true;
                    
                    // Send idle notification
                    let minutes = idle_time.as_secs() / 60;
                    let _ = send_idle_notification(&webhook, minutes);
                    
                    log::info!("System idle for {} minutes", minutes);
                }
                // If we were idle but now there's activity
                else if idle_time < idle_threshold && was_idle {
                    was_idle = false;
                    
                    // Send active notification
                    let _ = send_active_notification(&webhook, idle_time.as_secs() / 60);
                    
                    log::info!("System returned from idle state");
                }
            }
        });
        
        Ok(())
    }
    
    pub fn update_activity(&self) {
        let mut last_activity = self.last_activity.lock().unwrap();
        *last_activity = Instant::now();
    }
    
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
}

fn send_idle_notification(webhook: &WebhookSender, idle_minutes: u64) -> Result<()> {
    let additional_fields = vec![
        ("Idle Time".to_string(), format!("{} minutes", idle_minutes)),
    ];
    
    webhook.send(
        EventCategory::Idle,
        "System Idle",
        &format!("System has been idle for {} minutes", idle_minutes),
        additional_fields
    )
}

fn send_active_notification(webhook: &WebhookSender, idle_minutes: u64) -> Result<()> {
    let additional_fields = vec![
        ("Was Idle For".to_string(), format!("{} minutes", idle_minutes)),
    ];
    
    webhook.send(
        EventCategory::Idle,
        "System Active",
        "System has returned from idle state",
        additional_fields
    )
}
