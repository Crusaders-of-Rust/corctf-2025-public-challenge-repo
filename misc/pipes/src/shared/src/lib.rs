include!(concat!(env!("OUT_DIR"), "/pipes.rs"));
pub static PIPE_IN_LOCATION: &str = "/tmp/pipe1";
pub static PIPE_OUT_LOCATION: &str = "/tmp/pipe2";
pub static KEY_LOCATION: &str = "/tmp/key";
impl Copy for client_message::PlayerMove {}
