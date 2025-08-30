use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use icalendar::{CalendarDateTime, DatePerhapsTime};

pub fn date_perhaps_time_to_utc(dpt: &DatePerhapsTime) -> DateTime<Utc> {
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
