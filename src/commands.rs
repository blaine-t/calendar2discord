use poise::serenity_prelude as serenity;
use crate::config::{
    add_mapping, remove_mapping, list_mappings, update_default_mapping, 
    load_config
};
use std::fs;
use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Data {}

/// Add or update an event mapping
#[poise::command(slash_command)]
pub async fn add_event_mapping(
    ctx: Context<'_>,
    #[description = "Event name to map"] event: String,
    #[description = "Discord status message"] message: Option<String>,
    #[description = "Discord status emoji"] emoji: Option<String>,
) -> Result<(), Error> {
    match add_mapping(event.clone(), message.clone(), emoji.clone()) {
        Ok(()) => {
            let response = format!(
                "✅ Successfully added/updated mapping for event: `{}`\n{}{}",
                event,
                message.as_ref().map(|m| format!("Message: {}\n", m)).unwrap_or_default(),
                emoji.as_ref().map(|e| format!("Emoji: {}", e)).unwrap_or_default()
            );
            ctx.say(response).await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Failed to add mapping: {}", e)).await?;
        }
    }
    Ok(())
}

/// Remove an event mapping
#[poise::command(slash_command)]
pub async fn remove_event_mapping(
    ctx: Context<'_>,
    #[description = "Event name to remove"] event: String,
) -> Result<(), Error> {
    match remove_mapping(&event) {
        Ok(true) => {
            ctx.say(format!("✅ Successfully removed mapping for event: `{}`", event)).await?;
        }
        Ok(false) => {
            ctx.say(format!("⚠️ No mapping found for event: `{}`", event)).await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Failed to remove mapping: {}", e)).await?;
        }
    }
    Ok(())
}

/// List all event mappings
#[poise::command(slash_command)]
pub async fn list_event_mappings(ctx: Context<'_>) -> Result<(), Error> {
    match list_mappings() {
        Ok(mappings) => {
            if mappings.is_empty() {
                ctx.say("📝 No event mappings configured.").await?;
                return Ok(());
            }

            let mut response = String::from("📋 **Current Event Mappings:**\n\n");
            
            for mapping in mappings {
                response.push_str(&format!(
                    "**Event:** `{}`\n",
                    mapping.event
                ));
                
                if let Some(message) = &mapping.message {
                    response.push_str(&format!("  Message: {}\n", message));
                }
                
                if let Some(emoji) = &mapping.emoji {
                    response.push_str(&format!("  Emoji: {}\n", emoji));
                }
                
                response.push('\n');
            }

            // Discord has a 2000 character limit for messages
            if response.len() > 1900 {
                response.truncate(1900);
                response.push_str("\n... (truncated)");
            }
            
            ctx.say(response).await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Failed to list mappings: {}", e)).await?;
        }
    }
    Ok(())
}

/// Update default mapping settings
#[poise::command(slash_command)]
pub async fn update_default(
    ctx: Context<'_>,
    #[description = "Default Discord status message"] message: Option<String>,
    #[description = "Default Discord status emoji"] emoji: Option<String>,
) -> Result<(), Error> {
    if message.is_none() && emoji.is_none() {
        ctx.say("⚠️ Please provide at least one parameter (message or emoji) to update.").await?;
        return Ok(());
    }

    match update_default_mapping(message.clone(), emoji.clone()) {
        Ok(()) => {
            let response = format!(
                "✅ Successfully updated default settings:\n{}{}",
                message.as_ref().map(|m| format!("Default message: {}\n", m)).unwrap_or_default(),
                emoji.as_ref().map(|e| format!("Default emoji: {}", e)).unwrap_or_default()
            );
            ctx.say(response).await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Failed to update default settings: {}", e)).await?;
        }
    }
    Ok(())
}

/// Show current default settings
#[poise::command(slash_command)]
pub async fn show_default(ctx: Context<'_>) -> Result<(), Error> {
    match load_config() {
        Ok(config) => {
            let response = format!(
                "⚙️ **Current Default Settings:**\n\nMessage: `{}`\nEmoji: `{}`",
                if config.mappings.default.message.is_empty() {
                    "(none)"
                } else {
                    &config.mappings.default.message
                },
                if config.mappings.default.emoji.is_empty() {
                    "(none)"
                } else {
                    &config.mappings.default.emoji
                }
            );
            ctx.say(response).await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Failed to load config: {}", e)).await?;
        }
    }
    Ok(())
}

/// Upload a new calendar file
#[poise::command(slash_command)]
pub async fn upload_calendar(
    ctx: Context<'_>,
    #[description = "Calendar file to upload (.ics format)"] attachment: serenity::Attachment,
    #[description = "Name for the calendar file (without .ics extension)"] name: Option<String>,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    
    // Validate file extension
    if !attachment.filename.ends_with(".ics") {
        ctx.say("❌ Please upload a valid .ics calendar file.").await?;
        return Ok(());
    }
    
    // Create directory for user if it doesn't exist
    let user_dir = format!("calendars/{}", user_id);
    if let Err(e) = fs::create_dir_all(&user_dir) {
        ctx.say(format!("❌ Failed to create calendar directory: {}", e)).await?;
        return Ok(());
    }
    
    // Determine filename
    let filename = if let Some(custom_name) = name {
        // Sanitize the custom name and ensure .ics extension
        let sanitized = custom_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>();
        
        if sanitized.is_empty() {
            ctx.say("❌ Invalid calendar name. Please use only alphanumeric characters, underscores, or hyphens.").await?;
            return Ok(());
        }
        
        format!("{}.ics", sanitized)
    } else {
        // Use original filename
        attachment.filename.clone()
    };
    
    let file_path = format!("{}/{}", user_dir, filename);
    
    // Check if file already exists
    if Path::new(&file_path).exists() {
        ctx.say(format!("⚠️ A calendar with the name `{}` already exists. Please choose a different name or remove the existing calendar first.", filename)).await?;
        return Ok(());
    }
    
    // Download and validate the calendar file
    match attachment.download().await {
        Ok(content) => {
            // Basic validation - try to parse as ICS
            match std::str::from_utf8(&content) {
                Ok(content_str) => {
                    // Basic ICS format validation
                    if !content_str.contains("BEGIN:VCALENDAR") || !content_str.contains("END:VCALENDAR") {
                        ctx.say("❌ Invalid calendar file format. Please ensure it's a valid .ics file.").await?;
                        return Ok(());
                    }
                    
                    // Save the file
                    match fs::write(&file_path, content) {
                        Ok(()) => {
                            ctx.say(format!(
                                "✅ Successfully uploaded calendar `{}` to your calendar directory!\n📁 File saved as: `{}`",
                                filename,
                                file_path
                            )).await?;
                        }
                        Err(e) => {
                            ctx.say(format!("❌ Failed to save calendar file: {}", e)).await?;
                        }
                    }
                }
                Err(_) => {
                    ctx.say("❌ Invalid file encoding. Please ensure the calendar file is in UTF-8 format.").await?;
                }
            }
        }
        Err(e) => {
            ctx.say(format!("❌ Failed to download attachment: {}", e)).await?;
        }
    }
    
    Ok(())
}

/// List uploaded calendars
#[poise::command(slash_command)]
pub async fn list_calendars(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let user_dir = format!("calendars/{}", user_id);
    
    match fs::read_dir(&user_dir) {
        Ok(entries) => {
            let calendar_files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("ics"))
                .collect();
            
            if calendar_files.is_empty() {
                ctx.say("📅 No calendar files found. Use `/upload_calendar` to add one!").await?;
                return Ok(());
            }
            
            let mut response = String::from("📅 **Your Uploaded Calendars:**\n\n");
            
            for entry in calendar_files {
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();
                
                // Get file size
                if let Ok(metadata) = entry.metadata() {
                    let size_kb = metadata.len() / 1024;
                    response.push_str(&format!("• `{}` ({} KB)\n", filename_str, size_kb));
                } else {
                    response.push_str(&format!("• `{}`\n", filename_str));
                }
            }
            
            // Discord has a 2000 character limit for messages
            if response.len() > 1900 {
                response.truncate(1900);
                response.push_str("\n... (truncated)");
            }
            
            ctx.say(response).await?;
        }
        Err(_) => {
            ctx.say("📅 No calendar directory found. Use `/upload_calendar` to add your first calendar!").await?;
        }
    }
    
    Ok(())
}

/// Remove a calendar file
#[poise::command(slash_command)]
pub async fn remove_calendar(
    ctx: Context<'_>,
    #[description = "Name of the calendar file to remove (with .ics extension)"] filename: String,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    
    // Validate filename has .ics extension
    if !filename.ends_with(".ics") {
        ctx.say("❌ Please specify a calendar filename with the .ics extension.").await?;
        return Ok(());
    }
    
    let file_path = format!("calendars/{}/{}", user_id, filename);
    
    match fs::remove_file(&file_path) {
        Ok(()) => {
            ctx.say(format!("✅ Successfully removed calendar `{}`", filename)).await?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            ctx.say(format!("⚠️ Calendar file `{}` not found.", filename)).await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Failed to remove calendar: {}", e)).await?;
        }
    }
    
    Ok(())
}

/// Show help information about available commands
#[poise::command(slash_command)]
pub async fn help_command(ctx: Context<'_>) -> Result<(), Error> {
    let help_text = r#"
**Calendar2Discord Bot Commands**

**Calendar Management Commands:**
• `/upload_calendar` - Upload a new .ics calendar file
• `/list_calendars` - List all your uploaded calendars
• `/remove_calendar` - Remove a calendar file

**Event Mapping Commands:**
• `/add_event_mapping` - Add or update an event mapping
• `/remove_event_mapping` - Remove an event mapping
• `/list_event_mappings` - List all current mappings

**Default Settings Commands:**
• `/update_default` - Update default message/emoji
• `/show_default` - Show current default settings

**General:**
• `/help_command` - Show this help message

**How it works:**
The bot automatically checks your calendar and sets your Discord status based on current events. You can configure mappings for specific event names to customize your status message and emoji.
"#;
    ctx.say(help_text).await?;
    Ok(())
}

pub async fn start_discord_bot(token: String) -> Result<(), Error> {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                add_event_mapping(),
                remove_event_mapping(),
                list_event_mappings(),
                update_default(),
                show_default(),
                upload_calendar(),
                list_calendars(),
                remove_calendar(),
                help_command(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client?.start().await?;
    Ok(())
}
