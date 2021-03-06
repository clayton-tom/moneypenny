pub struct Message {
    pub body: String,
    pub output_time: bool,
    pub sender: String,
}

pub mod core_io {
    use std::fs;
        
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

    pub fn output_startup_message(state_config: super::core_config::Config) {
        let greeting = super::core_config::create_greeting_string_from_config(state_config);
        let greeting_msg = super::Message {
            body: greeting,
            output_time: true,
            sender: String::from("Core"),
        };
        output_message(greeting_msg);
    }

    fn print_to_output(output: String) {
        println!("{}", output);
    }

    pub fn read_file_as_str(path: String) -> String {
        let config_str = match fs::read_to_string(path) {
            Ok(input) => input,
            Err(e) => {
                let err_msg = super::Message {
                    body: String::from("Error reading file from string."),
                    output_time: true,
                    sender: String::from("Core"),
                };
                output_message(err_msg);
                return e.to_string();
            }
        };
        return config_str;
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
            let re = Regex::new(r#"\[Test module\] [A-Za-z]+ +[0-9]+ [0-9]+:[0-9]+:[0-9]+: This is another test string."#).unwrap();
            assert!(re.is_match(&format_message_to_str(message_time)));
        }

        #[test]
        fn test_read_file_as_str() {
            let test_path = String::from("src/test/test.toml");
            let expected: String = String::from("[Test]\ntest_key = \"Test string\"");
            assert_eq!(expected, read_file_as_str(test_path));
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

pub mod core_config {
    use toml;
    use serde::Deserialize;

    #[derive(Deserialize, PartialEq, Debug)]
    pub struct Config {
        pub name: String,
        pub greeting: String,
    }

    pub fn parse_config_file_to_struct(path: String) -> Config {
        let str_content = super::core_io::read_file_as_str(path);
        let config: Config = toml::from_str(&str_content).unwrap();
        return config;
    }

    pub fn create_greeting_string_from_config(config: Config) -> String {
        let greeting_string: String = format!("{}, {}", config.greeting, config.name);
        return greeting_string;
    }

    #[cfg(test)]
    mod core_config_tests {
        use super::*;

        fn create_test_config() -> Config {
            let test_config = Config {
                name: String::from("Tom"),
                greeting: String::from("I HAVE NO GREETING")
            };
            return test_config;
        }

        #[test]
        fn test_parse_config_file_to_struct() {
            let test_path = String::from("src/test/test_config.toml");
            let test_config = create_test_config();
            assert_eq!(test_config, parse_config_file_to_struct(test_path));
        }

        #[test]
        fn test_create_greeting_string_from_config() {
            let test_string = String::from("I HAVE NO GREETING, Tom");
            let test_config = create_test_config();
            assert_eq!(test_string, create_greeting_string_from_config(test_config));
        }
    }
}
