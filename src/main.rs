use calendar2discord::commands::start_discord_bot;
use calendar2discord::config::load_config;
use calendar2discord::connection::event_to_discord_status;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[tokio::main]
async fn main() {
    let last_status_was_default = Arc::new(AtomicBool::new(false));
    tokio::spawn(event_to_discord_status(
        true,
        last_status_was_default.clone(),
    ));

    println!("Starting Discord bot...");

    match load_config() {
        Ok(config) => {
            if let Err(e) = start_discord_bot(config.discord.token).await {
                eprintln!("Error running Discord bot: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            eprintln!("Make sure config.json exists and contains a valid Discord token.");
            std::process::exit(1);
        }
    }
}
