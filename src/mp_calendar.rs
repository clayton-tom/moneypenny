use ical;
use chrono::prelude::*;
use crate::mp_core;

enum EventStatus {
    Tentative,
    Confirmed,
    Cancelled
}

pub struct MpEvent {
    name: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    location: String,
    description: String,
    status: EventStatus
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
    use super::MpEvent;

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
        todo!
    }
}