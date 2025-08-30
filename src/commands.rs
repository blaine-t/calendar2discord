use poise::serenity_prelude as serenity;
use crate::config::{
    add_mapping, remove_mapping, list_mappings, update_default_mapping, 
    load_config
};

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

/// Show help information about available commands
#[poise::command(slash_command)]
pub async fn help_command(ctx: Context<'_>) -> Result<(), Error> {
    let help_text = r#"
**Calendar2Discord Bot Commands**

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
