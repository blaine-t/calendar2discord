use calendar2discord::calendar::get_current_event;
use calendar2discord::discord::set_discord_status;
use calendar2discord::models::Status;
use icalendar::Component;

#[tokio::main]
async fn main() {
    let event = get_current_event();
    if let Some(event) = event {
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
