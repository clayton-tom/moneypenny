use std::str::FromStr;
use std::cmp::Ordering;
use chrono::prelude::*;
use crate::mp_core;

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, Eq)]
pub struct MpEvent {
    // TODO consider making name and start time non optional
    name: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    location: Option<String>,
    description: Option<String>,
    status: Option<EventStatus>
}

impl MpEvent {
    fn cmp_start_time(&self, other: &MpEvent) -> Option<Ordering> {
        let lhs_time = match self.start_time {
            Some(time) => time,
            None => return Some(Ordering::Greater)
        };
        let rhs_time = match other.start_time {
            Some(time) => time,
            None => return Some(Ordering::Less)
        };
        return lhs_time.partial_cmp(&rhs_time);
    }

    fn cmp_end_time(&self, other: &MpEvent) -> Option<Ordering> {
        let lhs_time = match self.end_time {
            Some(time) => time,
            None => return Some(Ordering::Greater)
        };
        let rhs_time = match other.end_time {
            Some(time) => time,
            None => return Some(Ordering::Less)
        };
        return lhs_time.partial_cmp(&rhs_time);
    }

    // Assumes lhs start time before rhs start time
    fn ordered_has_overlap(&self, other: &MpEvent) -> bool {
        let lhs_end_time = match self.end_time {
            Some(time) => time,
            None => return false
        };
        let rhs_start_time = match other.start_time {
            Some(time) => time,
            None => return false
        };
        return lhs_end_time.ge(&rhs_start_time);
    }
}

impl PartialEq for MpEvent {
    // name and start time the same
    fn eq(&self, other: &MpEvent) -> bool {
        let lhs_name = match &self.name {
            Some(name) => name,
            None => return false
        };
        let rhs_name = match &other.name {
            Some(name) => name,
            None => return false
        };
        let lhs_start = match self.start_time {
            Some(time) => time,
            None => return false
        };
        let rhs_start = match other.start_time {
            Some(time) => time,
            None => return false
        };
        return (lhs_name == rhs_name) && (lhs_start == rhs_start);
    }
}

impl PartialOrd for MpEvent {
    // Use start time cmp function as default for ordering trait
    fn partial_cmp(&self, other: &MpEvent) -> Option<Ordering> {
        return MpEvent::cmp_start_time(self, other);
    }
}

impl Ord for MpEvent {
    fn cmp(&self, other: &MpEvent) -> Ordering {
        let ord_start = match MpEvent::cmp_start_time(self, other) {
            Some(Ordering::Equal) => Ordering::Equal,
            Some(Ordering::Greater) => return Ordering::Greater,
            Some(Ordering::Less) => return Ordering::Less,
            None => panic!()
        };
        // Assumes event with same start time but finishing later is chronologically 'before'
        let ord_end = match MpEvent::cmp_end_time(self, other) {
            Some(Ordering::Equal) => Ordering::Equal,
            Some(Ordering::Greater) => return Ordering::Less,
            Some(Ordering::Less) => return Ordering::Greater,
            None => panic!()
        };
        let ord_name = match self.name.cmp(&other.name) {
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less
        };
        if ord_start == Ordering::Equal && ord_end == Ordering::Equal && ord_name == Ordering::Equal {
            return Ordering::Equal;
        } else {
            output_mp_calendar_message(String::from("MpEvent Ord is breaking it's contract"));
            panic!();
        }
    }
}

#[cfg(test)]
mod calendar_mpevent_tests {
    use super::*;

    fn make_event(start_secs: i64, end_secs: i64) -> MpEvent {
        let dt1 = Utc.timestamp(start_secs, 0);
        let dt2 = Utc.timestamp(end_secs, 0);
        let this_event = MpEvent {
            name: None,
            start_time: Some(dt1),
            end_time: Some(dt2),
            location: None,
            description: None,
            status:None
        };
        return this_event;
    }

    #[test]
    fn test_cmp_start_time() {
        let event1 = make_event(100, 300);
        let event2 = make_event(400, 500);
        let event3 = make_event(100, 200);
        let cmp_1_2 = event1.cmp_start_time(&event2);
        assert_eq!(Ordering::Less, cmp_1_2.unwrap());
        let cmp_2_1 = event2.cmp_start_time(&event1);
        assert_eq!(Ordering::Greater, cmp_2_1.unwrap());
        let cmp_1_3 = event1.cmp_start_time(&event3);
        assert_eq!(Ordering::Equal, cmp_1_3.unwrap());
    }

    #[test]
    fn test_ordered_has_overlap() {
        let event1 = make_event(100, 300);
        let event2 = make_event(200, 400);
        let event3 = make_event(500, 600);
        let event4 = make_event(600, 700);
        let overlap_1_2 = event1.ordered_has_overlap(&event2);
        assert_eq!(true, overlap_1_2);
        let overlap_2_3 = event2.ordered_has_overlap(&event3);
        assert_eq!(false, overlap_2_3);
        let overlap_3_4 = event3.ordered_has_overlap(&event4);
        assert_eq!(true, overlap_3_4);
    }
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