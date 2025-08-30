use serde::{Deserialize, Serialize};
use std::{fs, sync::{atomic::AtomicBool, Arc}};
use crate::{connection, status::Status};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub discord: DiscordConfig,
    pub mappings: Mappings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mappings {
    #[serde(default)]
    pub default: Status,
    pub mapping: Vec<EventMapping>,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            message: String::new(),
            emoji: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventMapping {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    let config_content = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config_content)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config_json = serde_json::to_string_pretty(config)?;
    fs::write("config.json", config_json)?;

    // Force a refresh of the status in case the new config affects it
    tokio::spawn(connection::event_to_discord_status(false, Arc::new(AtomicBool::new(false))));
    
    Ok(())
}

pub fn add_mapping(event: String, message: Option<String>, emoji: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut config = load_config()?;
    
    // Check if mapping already exists and update it
    for existing_mapping in &mut config.mappings.mapping {
        if existing_mapping.event.to_lowercase() == event.to_lowercase() {
            if let Some(msg) = message {
                existing_mapping.message = Some(msg);
            }
            if let Some(em) = emoji {
                existing_mapping.emoji = Some(em);
            }
            save_config(&config)?;
            return Ok(());
        }
    }
    
    // If mapping doesn't exist, add a new one
    let new_mapping = EventMapping {
        event,
        message,
        emoji,
    };
    
    config.mappings.mapping.push(new_mapping);
    save_config(&config)?;
    Ok(())
}

pub fn remove_mapping(event: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut config = load_config()?;
    let original_len = config.mappings.mapping.len();
    
    config.mappings.mapping.retain(|mapping| {
        mapping.event.to_lowercase() != event.to_lowercase()
    });
    
    let removed = config.mappings.mapping.len() < original_len;
    if removed {
        save_config(&config)?;
    }
    
    Ok(removed)
}

pub fn list_mappings() -> Result<Vec<EventMapping>, Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config()?;
    Ok(config.mappings.mapping.clone())
}

pub fn update_default_mapping(message: Option<String>, emoji: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut config = load_config()?;
    
    if let Some(msg) = message {
        config.mappings.default.message = msg;
    }
    if let Some(em) = emoji {
        config.mappings.default.emoji = em;
    }
    
    save_config(&config)?;
    Ok(())
}

pub fn map_event_to_status(event_name: &str) -> Status {
    let config = match load_config() {
        Ok(config) => config,
        Err(_) => {
            // Fallback to default if config loading fails
            return Status {
                message: String::new(),
                emoji: String::new(),
            };
        }
    };

    // Look for exact match in mappings
    for mapping in &config.mappings.mapping {
        if mapping.event.to_lowercase() == event_name.to_lowercase() {
            let message = mapping.message.as_ref()
                .unwrap_or(&config.mappings.default.message)
                .clone();
            let emoji = mapping.emoji.as_ref()
                .unwrap_or(&config.mappings.default.emoji)
                .clone();
            
            return Status { message, emoji };
        }
    }

    // Look for partial match (if event name contains the mapping event name)
    for mapping in &config.mappings.mapping {
        if event_name.to_lowercase().contains(&mapping.event.to_lowercase()) {
            let message = mapping.message.as_ref()
                .unwrap_or(&config.mappings.default.message)
                .clone();
            let emoji = mapping.emoji.as_ref()
                .unwrap_or(&config.mappings.default.emoji)
                .clone();
            
            return Status { message, emoji };
        }
    }

    // Return default if no match found
    Status {
        message: config.mappings.default.message,
        emoji: config.mappings.default.emoji,
    }
}
