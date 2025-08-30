use icalendar::{Calendar, CalendarComponent, Component, Event};
use std::fs::read_to_string;

use crate::util::date_perhaps_time_to_utc;

pub fn get_current_event() -> Option<Event> {
    let contents = read_to_string("test.ics").unwrap();

    let parsed_calendar: Calendar = contents.parse().unwrap();

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
                return Some(event.clone());
            }
        }
    }
    None
}
