use anyhow::Result;
use crate::webhook::{EventCategory, WebhookSender};

pub mod system;
pub mod usb;
pub mod idle;
pub mod heartbeat;
