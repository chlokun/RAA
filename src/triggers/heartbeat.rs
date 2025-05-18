use anyhow::Result;
use tokio_cron_scheduler::{JobScheduler, Job};
use std::sync::Arc;
use crate::webhook::{EventCategory, WebhookSender};
use crate::triggers::system;

pub struct HeartbeatScheduler {
    webhook: WebhookSender,
    interval_minutes: u64,
}

impl HeartbeatScheduler {
    pub fn new(webhook: WebhookSender, interval_minutes: u64) -> Self {
        Self {
            webhook,
            interval_minutes,
        }
    }
    
    pub async fn start(&self) -> Result<()> {
        let webhook = self.webhook.clone();
        let scheduler = JobScheduler::new().await?;
        
        // Define cron expression for the interval
        // This is every N minutes
        let cron_expr = format!("0 */{} * * * *", self.interval_minutes);
        
        // Create a job that will execute the heartbeat function
        let job = Job::new_async(cron_expr.as_str(), move |_, _| {
            let webhook_clone = webhook.clone();
            Box::pin(async move {
                send_heartbeat(&webhook_clone).unwrap_or_else(|e| {
                    log::error!("Failed to send heartbeat: {}", e);
                });
            })
        })?;
        
        // Add the job to the scheduler
        scheduler.add(job).await?;
        
        // Start the scheduler
        scheduler.start().await?;
        
        log::info!("Heartbeat scheduled to run every {} minutes", self.interval_minutes);
        
        // The scheduler will run on its own thread, so we can just return
        Ok(())
    }
}

fn send_heartbeat(webhook: &WebhookSender) -> Result<()> {
    // Get system information for the heartbeat
    let system_info = system::get_system_info();
    
    webhook.send(
        EventCategory::System,
        "Heartbeat",
        "Regular system heartbeat check-in",
        system_info
    )
}
