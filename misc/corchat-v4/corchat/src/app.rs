use std::{sync::Arc, time::Duration};

use crate::db::{self, Channel, Message};
use anyhow::{Result, bail};
use tokio::sync::{
    RwLock,
    mpsc::{self, Receiver, Sender},
};

pub struct App {
    username: String,
    active_channel: RwLock<Option<ActiveChannel>>,
    channels: RwLock<Vec<Channel>>,
}

impl App {
    pub async fn run(username: String) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<Command>(100);
        let (waker_tx, waker_rx) = mpsc::channel::<()>(100);

        let app = Arc::new(App {
            username,
            active_channel: RwLock::new(None),
            channels: RwLock::new(db::get_channels().await?),
        });
        std::thread::spawn(move || {
            input_handler(tx);
        });
        let app2 = Arc::clone(&app);
        tokio::spawn(async move { app2.message_handler(waker_rx).await });

        loop {
            Arc::clone(&app).handle_command(&waker_tx, &mut rx).await?
        }
    }

    // Query and print incoming messages
    async fn message_handler(self: Arc<Self>, mut waker: Receiver<()>) -> ! {
        loop {
            let sleep_fut = tokio::time::sleep(Duration::from_millis(50));
            let waker_fut = waker.recv();
            tokio::pin!(sleep_fut);
            tokio::pin!(waker_fut);
            futures::future::select(sleep_fut, waker_fut).await;
            if let Some(active) = self.active_channel.write().await.as_mut() {
                match active.get_messages().await {
                    Ok(messages) => {
                        if messages.is_empty() {
                            continue;
                        }
                        let to_print = messages
                            .iter()
                            .map(|msg| format!("[{}]: {}", msg.user(), msg.content()))
                            .collect::<Vec<_>>()
                            .join("\n");
                        println!("{to_print}");
                    }
                    Err(e) => {
                        eprintln!("failed to get messages: {e}");
                        continue;
                    }
                };
            }
        }
    }

    // Receive and process the latest command
    async fn handle_command(
        self: Arc<Self>,
        waker: &Sender<()>,
        commands: &mut Receiver<Command>,
    ) -> Result<()> {
        let Some(command) = commands.recv().await else {
            bail!("input thread closed")
        };
        match command {
            Command::EnterChannel(name) => {
                let channels = self.channels.read().await;
                let Some(channel) = channels.iter().find(|c| c.name() == name) else {
                    eprintln!("[SYS]: Channel `{name}` does not exist");
                    return Ok(());
                };
                *self.active_channel.write().await = Some(ActiveChannel {
                    channel: channel.clone(),
                    last_message: None,
                });
                Ok(())
            }
            Command::Leave => {
                *self.active_channel.write().await = None;
                Ok(())
            }
            Command::Line(line) => {
                if line.is_empty() {
                    return Ok(());
                }
                match self.active_channel.read().await.as_ref() {
                    Some(channel) => {
                        channel.send_message(&self.username, &line).await?;
                        // Immediately wake up message printer
                        waker.send(()).await?;
                    }
                    None => {
                        eprintln!("[SYS]: Please enter a channel before sending a message");
                    }
                }
                Ok(())
            }
        }
    }
}

fn input_handler(tx: Sender<Command>) -> ! {
    let stdin = std::io::stdin();
    loop {
        let mut buffer = String::new();
        if stdin.read_line(&mut buffer).is_err() {
            continue;
        }
        if !buffer.is_ascii() {
            continue;
        }
        buffer = buffer.trim().to_string();
        let _ = tx.blocking_send(Command::parse(buffer));
    }
}

#[derive(Clone, Debug)]
enum Command {
    EnterChannel(String),
    Leave,
    Line(String),
}

impl Command {
    pub fn parse(from: String) -> Self {
        match from.split_once(" ") {
            Some(("enter", channel)) if !channel.is_empty() => {
                Command::EnterChannel(channel.to_string())
            }
            _ if from == "LEAVE" => Command::Leave,
            _ => Command::Line(from),
        }
    }
}

#[derive(Clone)]
struct ActiveChannel {
    channel: Channel,
    // ID of last seen message
    last_message: Option<i32>,
}

impl ActiveChannel {
    async fn send_message(&self, username: &str, content: &str) -> Result<()> {
        self.channel.send_message(username, content).await
    }

    /// Get all messages seen since the last, and update the last seen one
    async fn get_messages(&mut self) -> Result<Vec<Message>> {
        let messages = self.channel.get_messages(self.last_message).await?;
        if !messages.is_empty() {
            self.last_message = messages.iter().map(|m| m.id()).max();
        }
        Ok(messages)
    }
}
