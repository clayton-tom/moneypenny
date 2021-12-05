use std::str::FromStr;
use std::cmp::Ordering;
use chrono::prelude::*;
use crate::mp_core;

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, Eq, Clone)]
pub struct MpEvent {
    // TODO consider making name and start time non optional
    name: Option<String>,
    start_time: Option<DateTime<FixedOffset>>,
    end_time: Option<DateTime<FixedOffset>>,
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

    /// Assumes lhs start time before rhs start time
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

    /// Convert an MpEvent into a string for it's ICS notation
    fn deserialise_to_ics_string(&self) -> String {
        let mut ics_event = String::from("BEGIN:VEVENT\n");
        let MpEvent {name, start_time, end_time, location, description, status} = self;
        match name {
            Some(name) => {
                let strapp = &format!("SUMMARY:{}\n", name);
                ics_event.push_str(&strapp)
            },
            None => {}
        };
        match start_time {
            Some(time_utc) => {
                let time = cal_io::convert_fixed_offset_to_ical_time(*time_utc);
                ics_event.push_str(&format!("DTSTART:{}\n", time));
            },
            None => ()
        };
        match end_time {
            Some(time_utc) => {
                let time = cal_io::convert_fixed_offset_to_ical_time(*time_utc);
                ics_event.push_str(&format!("DTEND:{}\n", time));
            },
            None => ()
        };
        match location {
            Some(loc) => { ics_event.push_str(&format!("LOCATION:{}\n", loc)) },
            None => ()
        };
        match description {
            Some(desc) => { ics_event.push_str(&format!("DESCRIPTION:{}\n", desc)) },
            None => ()
        };
        match status {
            Some(enum_status) => {
                let status;
                match enum_status {
                    EventStatus::Tentative => status = String::from("TENTATIVE"),
                    EventStatus::Confirmed => status = String::from("CONFIRMED"),
                    EventStatus::Cancelled => status = String::from("CANCELLED")
                }
                ics_event.push_str(&format!("STATUS:{}\n", status)) },
            None => ()
        };
        ics_event.push_str(&String::from("END:VEVENT\n"));
        return ics_event;
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
        // Assumes event with same start time but later finish is chronologically 'later'
        let ord_end = match MpEvent::cmp_end_time(self, other) {
            Some(Ordering::Equal) => Ordering::Equal,
            Some(Ordering::Greater) => return Ordering::Greater,
            Some(Ordering::Less) => return Ordering::Less,
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
        let dt1 = FixedOffset::west(0).timestamp(start_secs, 0);
        let dt2 = FixedOffset::west(0).timestamp(end_secs, 0);
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
    fn test_cmp_end_time() {
        let event1 = make_event(100, 300);
        let event2 = make_event(100, 500);
        let event3 = make_event(200, 300);
        let cmp_1_2 = event1.cmp_end_time(&event2);
        assert_eq!(Ordering::Less, cmp_1_2.unwrap());
        let cmp_2_1 = event2.cmp_end_time(&event1);
        assert_eq!(Ordering::Greater, cmp_2_1.unwrap());
        let cmp_1_3 = event1.cmp_end_time(&event3);
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
    use std::io::prelude::*;
    use std::io::Error;
    use std::fs::File;
    use super::{DateTime, Utc, FixedOffset, TimeZone}; // Chrono imports
    use super::{MpEvent, EventStatus, FromStr}; // MP imports

    pub fn parse_file_to_ical_calendar(path: String) -> Result<IcalCalendar, ical::parser::ParserError> {
        use std::io::BufReader;
        //use std::fs::File;

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

    /// This will create a new file to write to, or COMPLETELY OVERWRITE an existing one. Refactor for poss append?
    pub fn deserialise_mpevents_to_ics_file(write_path: String, events: Vec<MpEvent>) { // todo return
        use std::path::Path;

        let path = Path::new(&write_path);
        let display = path.display(); // gives string of filename

        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => {
                super::output_mp_calendar_message(format!("Couldn't create {}: {}", display, why));
            },
            Ok(mut open_file) => {
                let cal_start = "BEGIN:VCALENDAR\nVERSION:2.0\nCALSCALE:GREGORIAN\n"; // TODO take these elsewhere
                let cal_end = "END:VCALENDAR";
                open_file.write(cal_start.as_bytes()).unwrap();
                for event in events{
                    let event_string = event.deserialise_to_ics_string();
                    match open_file.write_all(event_string.as_bytes()) {
                        Ok(_) => {}, // continue
                        Err(e) => super::output_mp_calendar_message(format!("Failed to deserialise MPEvents to ICS file {}: {}", display, e))
                    }
                }
                open_file.write(cal_end.as_bytes()).unwrap();
            }
        };
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
                    mp_event.start_time = convert_ical_time_to_fixed_offset(prop.value, prop.params);
                } else if name == "DTEND" {
                    mp_event.end_time = convert_ical_time_to_fixed_offset(prop.value, prop.params);
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

    fn convert_ical_time_to_fixed_offset(ical_time: Option<String>, ical_tz: Option<Vec<(String, Vec<String>)>>) -> Option<DateTime<FixedOffset>> {
        match ical_tz {
            Some(tz_vec) => {
                // TODO
                // do something with timezone offset
            }
            None => ()
        }
        match ical_time {
            Some(ical_time) => {
                let offset = 0; // todo
                let split_vec: Vec<&str> = ical_time.split("T").collect();
                let date_str = split_vec[0]; // e.g. 20130802
                let time_str = split_vec[1]; // e.g. 200000(Z)
                let (year_str, month_str, day_str) = (&date_str[..4], &date_str[4..6], &date_str[6..8]);
                let (hr_str, min_str, sec_str) = (&time_str[..2], &time_str[2..4], &time_str[4..6]);
                let dt_str = format!("{} {} {} {} {} {}", year_str, month_str, day_str, hr_str, min_str, sec_str);
                let format_str = String::from("%Y %m %d %H %M %S");
                let time: Result<DateTime<FixedOffset>, _> = FixedOffset::west(offset).datetime_from_str(&dt_str, &format_str);
                match time {
                    Ok(time_fo) => return Some(time_fo),
                    Err(e) => {
                        super::output_mp_calendar_message(format!("Error converting Ical time to FixedOffset: {}", e.to_string()));
                        return None
                    }
                }
            }
            None => return None
        }
    }

    /// Converts fixed offset time to string ical format YYYYMMDD'T'HHMMSS
    pub fn convert_fixed_offset_to_ical_time(fo_time: DateTime<FixedOffset>) -> String {
        let format = String::from("%Y%m%dT%H%M%S");
        let ical_str = format!("{}", fo_time.format(&format));
        return ical_str;
    }

    #[cfg(test)]
    mod cal_io_tests {
        use crate::mp_calendar::cal_io::*;

        #[test]
        fn test_convert_ical_time_to_fixed_offset() {
            let test_ical_time = Some(String::from("20130802T200000Z"));
            let test_ical_tz_vec = None;
            let test_time_fixed_offset = convert_ical_time_to_fixed_offset(test_ical_time, test_ical_tz_vec).unwrap();
            let expected_time_fixed_offset = DateTime::parse_from_rfc3339(&String::from("2013-08-02T20:00:00-00:00")).unwrap();
            assert_eq!(expected_time_fixed_offset, test_time_fixed_offset);
        }

        #[test]
        fn test_convert_fixed_offset_to_ical_time() {
            let time_fixedoff = DateTime::parse_from_rfc3339(&String::from("2013-08-02T20:00:00-00:00")).unwrap();
            let expected_ical_time = String::from("20130802T200000");
            let test_ical_time = convert_fixed_offset_to_ical_time(time_fixedoff);
            assert_eq!(expected_ical_time, test_ical_time);
        }
    }
}

pub mod cal_ops {
    use super::{MpEvent, DateTime, FixedOffset, EventStatus};

    /// By default .sort() uses partial_cmp, this uses cmp for comparison by total ordering (Ord not PartialOrd)
    pub fn sort_mpevents_chronologically_by_start(mut events: Vec<MpEvent>) -> Vec<MpEvent> {
        events.sort_by(|a, b| a.cmp(b));
        return events;
    }

    /// Creates a new MPEvent from a series of inputs
    fn create_new_MPEvent(name: Option<String>,
                        start_time: Option<DateTime<FixedOffset>>,
                        end_time: Option<DateTime<FixedOffset>>,
                        location: Option<String>,
                        description: Option<String>,
                        status: Option<EventStatus>) -> MpEvent {
        return MpEvent{ name, start_time, end_time, location, description, status };
    }

    pub mod cal_ops_tests {
        use crate::mp_calendar::cal_ops::*;

        #[test]
        pub fn test_sort_mpevents_chronologically_by_start() {
            let time_1 = DateTime::parse_from_rfc3339(&String::from("2013-08-02T20:00:00-00:00")).unwrap();
            let time_2 = DateTime::parse_from_rfc3339(&String::from("2013-08-03T20:00:00-00:00")).unwrap();
            let time_3 = DateTime::parse_from_rfc3339(&String::from("2013-08-03T22:00:00-00:00")).unwrap();
            let time_4 = DateTime::parse_from_rfc3339(&String::from("2013-08-04T22:00:00-00:00")).unwrap();
            let event_1 = MpEvent {name: Some(String::from("Event1")),  start_time: Some(time_1), end_time: None, description: None, location: None, status: None };
            let event_2 = MpEvent {name: Some(String::from("Event2")),  start_time: Some(time_2), end_time: Some(time_3), description: None, location: None, status: None };
            let event_3 = MpEvent {name: Some(String::from("Event3")),  start_time: Some(time_2), end_time: Some(time_4), description: None, location: None, status: None };
            let event_4 = MpEvent {name: Some(String::from("Event4")),  start_time: Some(time_3), end_time: None, description: None, location: None, status: None };
            let mut unsorted_events: Vec<MpEvent> = vec!(event_3.clone(), event_2.clone(), event_4.clone(), event_1.clone());
            let exp_sorted_events: Vec<MpEvent> = vec!(event_1, event_2, event_3, event_4);
            let sorted_events = sort_mpevents_chronologically_by_start(unsorted_events);
            assert_eq!(exp_sorted_events, sorted_events);
        }
    }
}