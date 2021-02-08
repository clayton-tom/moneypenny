use ical;
use std::str::FromStr;
use chrono::prelude::*;
use crate::mp_core;

#[derive(Debug)]
enum EventStatus {
    Tentative,
    Confirmed,
    Cancelled
}

impl FromStr for EventStatus {
    type Err = ();
    fn from_str(input: &str) -> Result<EventStatus, Self::Err> {
        match input {
            "TENTATIVE" => Ok(EventStatus::Tentative),
            "CONFIRMED" => Ok(EventStatus::Confirmed),
            "CANCELLED" => Ok(EventStatus::Cancelled),
            _           => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct MpEvent {
    name: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    location: Option<String>,
    description: Option<String>,
    status: Option<EventStatus>
}

fn output_mp_calendar_message(str_message: String) {
    let message = mp_core::Message {
        body: str_message,
        output_time: true,
        sender: String::from("Calendar")
    };
    mp_core::core_io::output_message(message);
}

pub mod cal_io {
    use ical::parser::ical::component::IcalCalendar;
    use super::DateTime;
    use super::Utc;
    use super::{MpEvent, EventStatus, FromStr};

    pub fn parse_file_to_ical_calendar(path: String) -> Result<IcalCalendar, ical::parser::ParserError> {
        use std::io::BufReader;
        use std::fs::File;

        let buf = BufReader::new(File::open(path).unwrap());
        let mut reader = ical::IcalParser::new(buf);
        
        let cal_option = reader.next();
        match cal_option {
            Some(cal_result) => {
                super::output_mp_calendar_message(String::from("ICalParser successfully read from file"));
                return cal_result;
            }
            None => {
                super::output_mp_calendar_message(String::from("ICalParser could not read from file"));
                return Err(ical::parser::ParserError::NotComplete);
            }
        }
    }

    pub fn extract_events_from_ical(cal: IcalCalendar) -> Vec<MpEvent> {
        let events = cal.events;
        let mut mp_events: Vec<MpEvent> = vec![];
        for event in events {
            let mut mp_event = MpEvent {
                name: None,
                start_time: None,
                end_time: None,
                location: None,
                description: None,
                status: None
            };
            let event_props = event.properties;
            for prop in event_props {
                let name = prop.name;
                if name == "SUMMARY" {
                    mp_event.name = prop.value;
                } else if name == "DTSTART" {
                    mp_event.start_time = convert_ical_time_to_utc(prop.value, prop.params);
                } else if name == "DTEND" {
                    mp_event.end_time = convert_ical_time_to_utc(prop.value, prop.params);
                } else if name == "LOCATION" {
                    mp_event.location = prop.value;
                } else if name == "DESCRIPTION" {
                    mp_event.description = prop.value;
                } else if name == "STATUS" {
                    match prop.value {
                        Some(str) => {
                            mp_event.status = Some(EventStatus::from_str(&str).unwrap());
                        }
                        // TODO: assuming an event without a status is tentative may not be the best call. Maybe leave at None?
                        None => mp_event.status = Some(EventStatus::Tentative)
                    }
                }
            }
            mp_events.push(mp_event);
        }
        return mp_events;
    }

    fn convert_ical_time_to_utc(ical_time: Option<String>, ical_tz: Option<Vec<(String, Vec<String>)>>) -> Option<DateTime<Utc>> {
        todo!();
    }
}