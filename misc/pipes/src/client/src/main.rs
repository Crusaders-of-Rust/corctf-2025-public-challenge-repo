use anyhow::{Context, Result};
use prost::Message as ProstMessage;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::text::Line;
use ratatui::{Terminal, backend::CrosstermBackend, style::*, widgets::*};
use shared::{
    ClientMessage, ServerMessage,
    client_message::{PlayerMove, player_move::Direction},
};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::pipe::{self, Receiver, Sender};

async fn load_key() -> Result<[u8; 16]> {
    let mut buf = [0; 16];
    let mut file = OpenOptions::new()
        .read(true)
        .create(false)
        .open(shared::KEY_LOCATION)
        .await
        .context("reading keyfile")?;
    file.read_exact(&mut buf).await?;
    Ok(buf)
}

async fn open_pipe() -> Result<(Sender, Receiver)> {
    let sender = pipe::OpenOptions::new()
        .read_write(true)
        .open_sender(shared::PIPE_IN_LOCATION)
        .context("opening named pipe for sending")?;
    let receiver = pipe::OpenOptions::new()
        .read_write(true)
        .open_receiver(shared::PIPE_OUT_LOCATION)
        .context("opening named pipe for receiving")?;
    Ok((sender, receiver))
}

fn render_maze_ui<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    maze_state: &str,
    flag: Option<&str>,
) -> Result<()> {
    terminal.draw(|f| {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title("Maze Game".to_string());
        if let Some(flag) = flag {
            block = block.title_top(Line::from(flag).right_aligned());
        }
        let maze_widget = Paragraph::new(maze_state)
            .block(block)
            .style(Style::default().fg(Color::White));
        let area = f.area();
        f.render_widget(maze_widget, area);
    })?;
    Ok(())
}

async fn read_server_msg(receiver: &mut Receiver) -> Result<Option<ServerMessage>> {
    let mut buf = vec![0; 4096 * 4];
    let n = match receiver.try_read(&mut buf) {
        Ok(0) => return Ok(None),
        Ok(n) => n,
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
        Err(e) => Err(e)?,
    };
    Ok(Some(ServerMessage::decode(&buf[..n])?))
}

async fn send_message(sender: &mut Sender, message: ClientMessage) -> Result<()> {
    sender.write_all(&message.encode_to_vec()).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let protocol_key = load_key().await?.to_vec();
    let (mut pipe_sender, mut pipe_receiver) = open_pipe().await?;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    send_message(
        &mut pipe_sender,
        ClientMessage {
            key: protocol_key.clone(),
            player_move: None,
            request_maze_state: Some(true),
        },
    )
    .await?;
    let mut maze_state = String::new();
    let mut flag = None;
    while flag.is_none() {
        if let Some(response) = read_server_msg(&mut pipe_receiver).await? {
            maze_state = response.maze_state;
            if let Some(rflag) = response.flag {
                flag.get_or_insert_with(|| rflag.clone());
            }
        }

        render_maze_ui(&mut terminal, &maze_state, flag.as_deref())?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                let direction = match key.code {
                    KeyCode::Char('w') | KeyCode::Up => Direction::Up,
                    KeyCode::Char('a') | KeyCode::Left => Direction::Left,
                    KeyCode::Char('s') | KeyCode::Down => Direction::Down,
                    KeyCode::Char('d') | KeyCode::Right => Direction::Right,
                    KeyCode::Char('q') => break,
                    _ => continue,
                };

                send_message(
                    &mut pipe_sender,
                    ClientMessage {
                        key: protocol_key.clone(),
                        player_move: Some(PlayerMove {
                            direction: direction.into(),
                            amount: 1,
                        }),
                        request_maze_state: Some(true),
                    },
                )
                .await?;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    Ok(())
}
