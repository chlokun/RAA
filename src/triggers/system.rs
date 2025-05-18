use anyhow::Result;
use sysinfo::{System, SystemExt};
use crate::webhook::{EventCategory, WebhookSender};

pub fn send_boot_notification(webhook: &WebhookSender) -> Result<()> {
    // Get system information
    let mut system = System::new_all();
    system.refresh_all();
    
    let os_version = format!(
        "{} {}",
        system.name().unwrap_or_else(|| "Unknown OS".to_string()),
        system.os_version().unwrap_or_else(|| "Unknown version".to_string())
    );
    
    let kernel_version = system.kernel_version().unwrap_or_else(|| "Unknown kernel".to_string());
    let hostname = system.host_name().unwrap_or_else(|| "Unknown hostname".to_string());
    let uptime = system.uptime();
    
    let additional_fields = vec![
        ("OS".to_string(), os_version),
        ("Kernel".to_string(), kernel_version),
        ("Host".to_string(), hostname),
        ("Uptime".to_string(), format!("{} seconds", uptime)),
    ];
    
    webhook.send(
        EventCategory::System,
        "System Started",
        "The system has been started or RAA has been launched.",
        additional_fields
    )
}

pub fn get_system_info() -> Vec<(String, String)> {
    let mut system = System::new_all();
    system.refresh_all();
    
    vec![
        ("OS".to_string(), system.name().unwrap_or_else(|| "Unknown".to_string())),
        ("OS Version".to_string(), system.os_version().unwrap_or_else(|| "Unknown".to_string())),
        ("Kernel".to_string(), system.kernel_version().unwrap_or_else(|| "Unknown".to_string())),
        ("Hostname".to_string(), system.host_name().unwrap_or_else(|| "Unknown".to_string())),
        ("Total Memory".to_string(), format!("{} MB", system.total_memory() / 1024)),
        ("Used Memory".to_string(), format!("{} MB", system.used_memory() / 1024)),
        ("CPU Count".to_string(), system.cpus().len().to_string()),
    ]
}
