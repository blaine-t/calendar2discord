use calendar2discord::calendar::get_current_event;
use calendar2discord::discord::set_discord_status;
use calendar2discord::mapping::map_event_to_status;
use icalendar::Component;

#[tokio::main]
async fn main() {
    let event = get_current_event();
    if let Some(event) = event {
        let event_name = event.get_summary().unwrap().to_string();
        let status = map_event_to_status(&event_name);
        
        println!("Event: {}", event_name);
        println!("Mapped to: {} {}", status.emoji, status.message);
        
        println!(
            "{:?}",
            set_discord_status(status).await
        );
    }
}
