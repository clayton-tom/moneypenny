pub struct Message {
    pub body: String,
    pub output_time: bool,
    pub sender: String,
}

pub mod core_io {
        
    pub fn output_message(message: super::Message) {
        let msg_string: String = format_message_to_str(message);
        print_to_output(msg_string)
    }

    fn format_message_to_str(message: super::Message) -> String {
        let msg_string: String;
        if message.output_time {
            let time_str = crate::mp_core::core_time::get_time_as_str();
            msg_string = format!("[{}] {}: {}", message.sender, time_str, message.body);
        } else {
            msg_string = format!("[{}] {}", message.sender, message.body);
        }
        return msg_string;
    }

    fn print_to_output(message: String) {
        println!("{}", message);
    }

    #[cfg(test)]
    mod core_io_tests {
        use crate::mp_core::Message;
        use crate::mp_core::core_io::*;
        use regex::Regex;

        #[test]
        fn test_format_message_to_str() {
            let message = Message {
                body: String::from("This is a test string."),
                output_time: false,
                sender: String::from("Test module"),
            };
            let expected_output = String::from("[Test module] This is a test string.");
            assert_eq!(expected_output, format_message_to_str(message));
            let message_time = Message {
                body: String::from("This is another test string."),
                output_time: true,
                sender: String::from("Test module"),
            };
            let re = Regex::new(r#"\[Test module\] [A-Za-z]+ [0-9]+ [0-9]+:[0-9]+:[0-9]+: This is another test string."#).unwrap();
            assert!(re.is_match(&format_message_to_str(message_time)));
        }
    }
}

mod core_time {
    use chrono::prelude::*;

    pub fn get_time_as_str() -> String {
        let local_time: DateTime<Local> = Local::now();
        let time = local_time.format("%b %e %T");
        let time_str = format!("{}", time);
        return time_str
    }
}
