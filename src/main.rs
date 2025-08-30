use std::fs::read_to_string;

use icalendar::{Calendar, CalendarComponent, Component};

struct Status {
    message: String,
    emoji: String
}

async fn set_discord_status(status: Status) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    
    let body = serde_json::json!({
        "custom_status": {
            "text": status.message,
            "emoji_name": status.emoji
        }
    }).to_string();

    let res = client.patch("https://discord.com/api/v10/users/@me/settings")
        .header("authorization", std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable not set"))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(reqwest::Error::from(res.error_for_status().unwrap_err()))
    }
}

#[tokio::main]
async fn main() {
    let contents = read_to_string("test.ics").unwrap();

    let parsed_calendar: Calendar = contents.parse().unwrap();

    let mut first = true;

    for component in &parsed_calendar.components {
        if let CalendarComponent::Event(event) = component {
            println!("Event: {}", event.get_summary().unwrap());
            if first {
                first = false;
                println!("{:?}", set_discord_status(Status { message: event.get_summary().unwrap().to_string(), emoji: "🤔".to_string() }).await);
            }
        }
    }
}
