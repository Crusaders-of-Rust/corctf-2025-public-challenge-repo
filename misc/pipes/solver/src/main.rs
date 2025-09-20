use std::time::Duration;

use evil_client_message::player_move::Direction;
use prost::Message as _;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

include!(concat!(env!("OUT_DIR"), "/pipes.rs"));

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pipe_path = "/tmp/pipe1";

    let mut file = OpenOptions::new().write(true).open(pipe_path).await?;

    let msg = EvilClientMessage {
        player_move: Some(evil_client_message::PlayerMove {
            direction: Direction::Right.into(),
            amount: 2,
        }),
    };

    let mut buf = vec![];
    loop {
        msg.encode(&mut buf).unwrap();
        file.write_all(&buf).await?;
        buf.truncate(0);
        std::thread::sleep(Duration::from_millis(1));
    }
}
