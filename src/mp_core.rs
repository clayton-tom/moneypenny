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
