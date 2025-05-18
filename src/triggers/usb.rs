use anyhow::Result;
use notify::{Watcher, RecursiveMode, EventKind, event::EventKind::*};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use crate::webhook::{EventCategory, WebhookSender};

#[cfg(target_os = "macos")]
const USB_PATH: &str = "/Volumes";

#[cfg(target_os = "windows")]
const USB_PATH: &str = ""; // Will use drive letters monitoring instead

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const USB_PATH: &str = "/media";

pub struct UsbMonitor {
    webhook: WebhookSender,
}

impl UsbMonitor {
    pub fn new(webhook: WebhookSender) -> Self {
        Self { webhook }
    }
    
    pub fn start_monitoring(&self) -> Result<()> {
        // Create a channel to receive the events
        let (tx, rx) = channel();
        
        // Create a watcher
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                tx.send(event).unwrap_or_else(|e| {
                    log::error!("Failed to send event through channel: {}", e);
                });
            }
        })?;
        
        // Start watching the USB path
        #[cfg(not(target_os = "windows"))]
        {
            // For macOS and Linux, watch the mounted volumes directory
            watcher.watch(Path::new(USB_PATH), RecursiveMode::NonRecursive)?;
        }
        
        #[cfg(target_os = "windows")]
        {
            // For Windows, we need to monitor drive letters
            // This is a simplified version and may need more robust implementation
            let drive_letters = get_windows_drive_letters();
            for drive in drive_letters {
                let path = format!("{}:\\", drive);
                if Path::new(&path).exists() {
                    let _ = watcher.watch(Path::new(&path), RecursiveMode::NonRecursive);
                }
            }
        }
        
        // Handle events
        std::thread::spawn(move || {
            for event in rx {
                match event.kind {
                    Create(_) => {
                        log::info!("USB device connected: {:?}", event.paths);
                        // Handle device connected event
                    },
                    Remove(_) => {
                        log::info!("USB device disconnected: {:?}", event.paths);
                        // Handle device disconnected event
                    },
                    _ => {}
                }
            }
        });
        
        Ok(())
    }
    
    pub fn send_usb_notification(&self, action: &str, device: &str) -> Result<()> {
        let additional_fields = vec![
            ("Action".to_string(), action.to_string()),
            ("Device".to_string(), device.to_string()),
        ];
        
        self.webhook.send(
            EventCategory::Usb,
            &format!("USB Device {}", action),
            &format!("USB device has been {}", action.to_lowercase()),
            additional_fields
        )
    }
}

#[cfg(target_os = "windows")]
fn get_windows_drive_letters() -> Vec<char> {
    // This is a simplified version, actual implementation would use Windows API
    ('A'..='Z').collect()
}
