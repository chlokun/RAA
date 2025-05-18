use anyhow::Result;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
struct WebhookPayload {
    username: String,
    content: String,
    #[serde(rename = "avatar_url")]
    avatar_url: Option<String>,
    embeds: Vec<WebhookEmbed>,
}

#[derive(Debug, Serialize)]
struct WebhookEmbed {
    title: String,
    description: Option<String>,
    color: u32,
    timestamp: String,
    fields: Vec<WebhookField>,
    footer: WebhookFooter,
}

#[derive(Debug, Serialize)]
struct WebhookField {
    name: String,
    value: String,
    inline: bool,
}

#[derive(Debug, Serialize)]
struct WebhookFooter {
    text: String,
}

pub enum EventCategory {
    System,
    Usb,
    Idle,
}

impl std::fmt::Display for EventCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventCategory::System => write!(f, "system"),
            EventCategory::Usb => write!(f, "usb"),
            EventCategory::Idle => write!(f, "idle"),
        }
    }
}

pub struct WebhookSender {
    client: reqwest::blocking::Client,
    device_name: String,
    webhooks: std::collections::HashMap<String, String>,
}

impl Clone for WebhookSender {
    fn clone(&self) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            device_name: self.device_name.clone(),
            webhooks: self.webhooks.clone(),
        }
    }
}

impl WebhookSender {
    pub fn new(device_name: String, system: String, usb: String, idle: String) -> Self {
        let client = reqwest::blocking::Client::new();
        let mut webhooks = std::collections::HashMap::new();
        
        webhooks.insert("system".to_string(), system);
        webhooks.insert("usb".to_string(), usb);
        webhooks.insert("idle".to_string(), idle);
        
        Self {
            client,
            device_name,
            webhooks,
        }
    }
    
    pub fn from_config(config: &crate::Config) -> Self {
        Self::new(
            config.device_name.clone(),
            config.webhooks.system.clone(),
            config.webhooks.usb.clone(),
            config.webhooks.idle.clone(),
        )
    }
    
    pub fn send(&self, category: EventCategory, title: &str, message: &str, additional_fields: Vec<(String, String)>) -> Result<()> {
        let category_str = category.to_string();
        
        if let Some(webhook_url) = self.webhooks.get(&category_str) {
            // Get current ISO timestamp
            let now = SystemTime::now();
            let timestamp = now.duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            let iso_time = chrono::DateTime::<chrono::Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap_or_default(),
                chrono::Utc,
            ).to_rfc3339();
            
            // Create fields for the embed
            let mut fields = Vec::new();
            
            // Add message as a field
            fields.push(WebhookField {
                name: "Message".to_string(),
                value: message.to_string(),
                inline: false,
            });
            
            // Add any additional fields
            for (name, value) in additional_fields {
                fields.push(WebhookField {
                    name,
                    value,
                    inline: true,
                });
            }
            
            // Create the webhook payload
            let payload = WebhookPayload {
                username: format!("RAA - {}", self.device_name),
                content: "".to_string(),
                avatar_url: Some("https://i.imgur.com/example.png".to_string()),
                embeds: vec![WebhookEmbed {
                    title: title.to_string(),
                    description: None,
                    color: match category {
                        EventCategory::System => 0x3498db, // Blue
                        EventCategory::Usb => 0xe74c3c,    // Red
                        EventCategory::Idle => 0xf1c40f,   // Yellow
                    },
                    timestamp: iso_time,
                    fields,
                    footer: WebhookFooter {
                        text: format!("RAA v{} | Device: {}", env!("CARGO_PKG_VERSION"), self.device_name),
                    },
                }],
            };
            
            // Send the webhook
            self.client.post(webhook_url)
                .json(&payload)
                .send()?;
                
            log::info!("Sent {} webhook: {}", category_str, title);
            Ok(())
        } else {
            log::error!("No webhook URL configured for category: {}", category_str);
            Err(anyhow::anyhow!("No webhook URL configured for category: {}", category_str))
        }
    }
}

// Add some missing dependencies to Cargo.toml
// chrono = "0.4"
            fields.push(WebhookField {
                name: "Message".to_string(),
                value: message.to_string(),
                inline: false,
            });
            
            // Add additional fields
            for (name, value) in additional_fields {
                fields.push(WebhookField {
                    name,
                    value,
                    inline: true,
                });
            }
            
            // Create the embed
            let embed = WebhookEmbed {
                title: title.to_string(),
                description: None,
                color: match category {
                    EventCategory::System => 0x5865F2, // Discord blurple
                    EventCategory::Usb => 0xEB459E,    // Pink
                    EventCategory::Idle => 0xFEE75C,   // Yellow
                },
                timestamp: iso_time,
                fields,
                footer: WebhookFooter {
                    text: format!("RAA Agent on {}", self.device_name),
                },
            };
            
            // Create the payload
            let payload = WebhookPayload {
                username: format!("RAA - {}", self.device_name),
                content: "".to_string(),
                avatar_url: Some("https://i.imgur.com/Xvt4F5W.png".to_string()),
                embeds: vec![embed],
            };
            
            // Send the webhook
            self.client.post(webhook_url)
                .json(&payload)
                .send()?;
            
            log::info!("Sent webhook to {}: {}", category_str, title);
            Ok(())
        } else {
            log::error!("No webhook URL configured for category: {}", category_str);
            Err(anyhow::anyhow!("No webhook URL configured for category: {}", category_str))
        }
    }
}
