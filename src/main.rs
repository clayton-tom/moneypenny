mod mp_core;

fn main() {
    let message = mp_core::Message {
        body: String::from("This is a test string."),
        output_time: true,
        sender: String::from("Test module"),
    };
    mp_core::core_io::output_message(message);
}
