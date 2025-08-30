use std::fs::read_to_string;

use icalendar::{Calendar, CalendarComponent, Component};

fn main() {
    let contents = read_to_string("test.ics").unwrap();

    let parsed_calendar: Calendar = contents.parse().unwrap();

    for component in &parsed_calendar.components {
        if let CalendarComponent::Event(event) = component {
            println!("Event: {}", event.get_summary().unwrap())
        }
    }
}
