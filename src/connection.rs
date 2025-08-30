use crate::calendar::get_current_event;
use crate::config::{load_config, map_event_to_status};
use crate::status::set_discord_status;
use crate::util::date_perhaps_time_to_utc;
use icalendar::Component;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

pub async fn event_to_discord_status(discord_user_id: u64, repeat: bool, last_status_was_default: Arc<AtomicBool>) {
    loop {
        println!("Looping!");
        let event = get_current_event(discord_user_id);
        let mut sleep_duration = Duration::from_secs(60);

        if let Some(event) = event {
            let event_name = event.get_summary().unwrap().to_string();
            let status = map_event_to_status(&event_name);

            println!("Event: {}", event_name);
            println!("Mapped to: {} {}", status.emoji, status.message);

            let now_time = chrono::Utc::now();
            let event_time = date_perhaps_time_to_utc(&event.get_end().unwrap());

            sleep_duration = (event_time - now_time)
                .to_std()
                .unwrap_or(Duration::from_secs(60));
            println!("{:?} eepy times", sleep_duration);

            println!("{:?}", set_discord_status(status).await);
            last_status_was_default.store(false, Ordering::Relaxed);
        } else {
            println!("No current event found.");
            // Set discord status to default
            // Only set default status if it wasn't already set
            if !last_status_was_default.load(Ordering::Relaxed) {
                let default_status = load_config().unwrap().mappings.default;
                println!("{:?}", set_discord_status(default_status).await);
                last_status_was_default.store(true, Ordering::Relaxed);
            }
        }

        if !repeat {
            break;
        }

        tokio::time::sleep(sleep_duration).await;
    }
}
