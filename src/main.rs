use calendar2discord::commands::start_discord_bot;
use calendar2discord::config::load_config;
use calendar2discord::connection::event_to_discord_status;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let last_status_was_default = Arc::new(AtomicBool::new(false));

    match load_config() {
        Ok(config) => {
            tokio::spawn(event_to_discord_status(
                config.discord.user_id,
                true,
                last_status_was_default.clone(),
            ));
        }
        Err(e) => {
            eprintln!("Couldn't load config... {e}");
        }
    }

    println!("Starting Discord bot...");
    let discord_bot_token =
        std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN environment variable not set");

    if let Err(e) = start_discord_bot(discord_bot_token).await {
        eprintln!("Error running Discord bot: {}", e);
        std::process::exit(1);
    }
}
