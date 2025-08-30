use std::fs::read_to_string;

use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use icalendar::{Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime};

struct Status {
    message: String,
    emoji: String,
}

fn date_perhaps_time_to_utc(dpt: &DatePerhapsTime) -> DateTime<Utc> {
    match dpt {
        DatePerhapsTime::DateTime(CalendarDateTime::Utc(dt)) => *dt,
        DatePerhapsTime::DateTime(CalendarDateTime::Floating(dt)) => {
            // Treat floating time as UTC
            Utc.from_utc_datetime(dt)
        }
        DatePerhapsTime::DateTime(CalendarDateTime::WithTimezone { date_time, tzid: _ }) => {
            // Convert to UTC (this might need timezone conversion)
            Utc.from_utc_datetime(date_time)
        }
        DatePerhapsTime::Date(date) => {
            // Convert date to datetime at 00:00:00 UTC
            let naive_datetime = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            Utc.from_utc_datetime(&naive_datetime)
        }
    }
}

async fn set_discord_status(status: Status) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "custom_status": {
            "text": status.message,
            "emoji_name": status.emoji
        }
    })
    .to_string();

    let res = client
        .patch("https://discord.com/api/v10/users/@me/settings")
        .header(
            "authorization",
            std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable not set"),
        )
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

    let mut first = false;
    let now = chrono::Utc::now();

    for component in &parsed_calendar.components {
        if let CalendarComponent::Event(event) = component {
            let start_time = date_perhaps_time_to_utc(&event.get_start().unwrap());
            let end_time = date_perhaps_time_to_utc(&event.get_end().unwrap());
            if start_time < now && end_time > now {
                println!(
                    "Event: {}. Starts at: {:?}",
                    event.get_summary().unwrap(),
                    start_time
                );
                if first {
                    first = false;
                    println!(
                        "{:?}",
                        set_discord_status(Status {
                            message: event.get_summary().unwrap().to_string(),
                            emoji: "🤔".to_string()
                        })
                        .await
                    );
                }
            }
        }
    }
}
