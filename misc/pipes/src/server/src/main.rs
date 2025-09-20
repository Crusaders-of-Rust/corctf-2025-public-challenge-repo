use std::collections::HashSet;

use anyhow::Context;
use anyhow::Result;
use nix::sys::stat::Mode;
use nix::sys::stat::umask;
use nix::unistd::mkfifo;
use prost::Message as _;
use rand::seq::SliceRandom;
use shared::{
    ClientMessage, ServerMessage, client_message::PlayerMove,
    client_message::player_move::Direction,
};
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::unix::pipe::{self, Receiver, Sender};

async fn initialize_key() -> Result<[u8; 16]> {
    let mut buf = [0; 16];
    let mut urandom = OpenOptions::new()
        .read(true)
        .open("/dev/urandom")
        .await
        .context("reading urandom")?;
    urandom.read_exact(&mut buf).await?;
    let keypath = shared::KEY_LOCATION;
    let mut keyfile = OpenOptions::new()
        .create(true)
        .mode(0o400)
        .write(true)
        .open(keypath)
        .await
        .context("opening keyfile")?;
    keyfile.write_all(&buf).await?;
    Ok(buf)
}

async fn open_pipe() -> Result<(Receiver, Sender)> {
    let pipe_in_path = shared::PIPE_IN_LOCATION;
    let pipe_out_path = shared::PIPE_OUT_LOCATION;

    let _ = fs::remove_file(pipe_in_path).await;
    let _ = fs::remove_file(pipe_out_path).await;

    let old_umask = umask(Mode::empty());
    mkfifo(
        pipe_in_path,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IWGRP | Mode::S_IWOTH,
    )?;
    mkfifo(
        pipe_out_path,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IWGRP | Mode::S_IWOTH,
    )?;
    umask(old_umask);

    let receiver = pipe::OpenOptions::new()
        .read_write(true)
        .open_receiver(pipe_in_path)
        .context("opening named pipe for reading")?;
    let sender = pipe::OpenOptions::new()
        .read_write(true)
        .open_sender(pipe_out_path)
        .context("opening named pipe for writing")?;

    Ok((receiver, sender))
}

struct Maze {
    width: usize,
    height: usize,
    player_pos: (usize, usize),
    end_pos: (usize, usize),
    walls: HashSet<(usize, usize)>,
}

impl Maze {
    fn generate(width: usize, height: usize, wall_count: usize) -> Self {
        let mut walls = HashSet::new();
        let mut rng = rand::rng();

        for y in 0..height {
            for x in 0..width {
                if x == 0 || y == 0 || x == width - 10 || y == height - 1 {
                    walls.insert((x, y));
                }
            }
        }

        let mut available_positions: Vec<(usize, usize)> = (1..height - 1)
            .flat_map(|y| (1..width - 10).map(move |x| (x, y)))
            .collect();

        available_positions.shuffle(&mut rng);

        for _ in 0..wall_count {
            if let Some(pos) = available_positions.pop() {
                walls.insert(pos);
            }
        }

        let player_pos = available_positions.pop().unwrap();
        let end_pos = (width - 2, height - 2);
        Self {
            width,
            height,
            player_pos,
            end_pos,
            walls,
        }
    }

    fn get_cell(&self, pos: (usize, usize)) -> char {
        if pos == self.player_pos {
            'P'
        } else if pos == self.end_pos {
            'E'
        } else if self.walls.contains(&pos) {
            '#'
        } else {
            ' '
        }
    }

    fn render(&self) -> String {
        let mut output = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                output.push(self.get_cell((x, y)));
            }
            output.push('\n');
        }
        output
    }

    fn move_player(&mut self, command: PlayerMove) {
        let (x, y) = self.player_pos;
        let distance = command.amount as usize;
        let new_pos: Option<_> = (|| {
            Some(match command.direction() {
                Direction::Up => (x, y.checked_sub(distance)?),
                Direction::Right => (x.checked_add(distance)?, y),
                Direction::Down => (x, y.checked_add(distance)?),
                Direction::Left => (x.checked_sub(distance)?, y),
            })
        })();
        match new_pos {
            Some(np) if self.get_cell(np) != '#' => self.player_pos = np,
            _ => (),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let flag = fs::read_to_string("/flag.txt")
        .await
        .unwrap_or_else(|_| "corctf{fake_flag_for_testing}".to_string());
    let key = initialize_key().await?;
    let (pipe_receiver, mut pipe_sender) = open_pipe().await?;

    let mut maze = Maze::generate(100, 30, 500);
    let mut reader = BufReader::new(pipe_receiver);
    let mut buf = vec![0; 1024];
    let mut won = false;
    let mut maze_state = maze.render();

    while !won {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            continue;
        }
        let Ok(msg) = ClientMessage::decode(&buf[..n]) else {
            continue;
        };
        if msg.key != key {
            continue;
        }

        if let Some(player_move) = msg.player_move {
            maze.move_player(player_move);
        }

        if msg.request_maze_state() {
            maze_state = maze.render();
        }

        won = maze.player_pos == maze.end_pos;
        let flag = won.then(|| flag.clone());
        let response = ServerMessage {
            flag,
            maze_state: maze_state.clone(),
        };
        let response_buf = response.encode_to_vec();
        pipe_sender.write_all(&response_buf).await?;
    }
    Ok(())
}
