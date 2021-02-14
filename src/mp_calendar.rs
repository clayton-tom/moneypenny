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
    use super::{DateTime, Utc, TimeZone}; // Chrono imports
    use super::{MpEvent, EventStatus, FromStr}; // MP imports

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
        // for now we're pretending all time is in UTC
        match ical_tz {
            Some(tz_vec) => {
                // TODO
                // do something with timezone offset
            }
            None => ()
        }
        match ical_time {
            Some(ical_time) => {
                let split_vec: Vec<&str> = ical_time.split("T").collect();
                let date_str = split_vec[0]; // e.g. 20130802
                let time_str = split_vec[1]; // e.g. 200000(Z)
                let (year_str, month_str, day_str) = (&date_str[..4], &date_str[4..6], &date_str[6..8]);
                let (hr_str, min_str, sec_str) = (&time_str[..2], &time_str[2..4], &time_str[4..6]);
                let dt_str = format!("{} {} {} {} {} {}", year_str, month_str, day_str, hr_str, min_str, sec_str);
                let format_str = String::from("%Y %m %d %H %M %S");
                let time: Result<DateTime<Utc>, _> = Utc::datetime_from_str(&Utc, &dt_str, &format_str);
                match time {
                    Ok(time_utc) => return Some(time_utc),
                    Err(e) => {
                        super::output_mp_calendar_message(format!("Error converting Ical time to UTC: {}", e.to_string()));
                        return None
                    }
                }
            }
            None => return None
        }
    }

    #[cfg(test)]
    mod cal_io_tests {
        use crate::mp_calendar::cal_io::*;

        #[test]
        fn test_convert_ical_time_to_utc_no_offset() {
            let test_ical_time = Some(String::from("20130802T200000Z"));
            let test_ical_tz_vec = None;
            let test_time_utc = convert_ical_time_to_utc(test_ical_time, test_ical_tz_vec).unwrap();
            let expected_time_fixedoff = DateTime::parse_from_rfc3339(&String::from("2013-08-02T20:00:00-00:00")).unwrap();
            assert_eq!(expected_time_fixedoff, test_time_utc);
            println!("{:?}", test_time_utc);
        }
    }

}