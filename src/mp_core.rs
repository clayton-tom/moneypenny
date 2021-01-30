pub mod core_io {
        
    pub fn output_message(message: &str) {
        let msg_string = String::from(message);
        println!("{}", msg_string);
    }

    pub fn output_message_with_time(message: &str) {
        let time_str = crate::mp_core::core_time::get_time_as_str();
        let msg_string = format!("{}: {}", time_str, message);
        output_message(&msg_string);
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
