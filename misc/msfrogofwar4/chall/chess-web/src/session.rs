use crate::engine::GameEngine;
use shakmaty::Chess;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum GameStatus {
    WaitingForPlayer,
    Running,
    Disconnected,
    Finished,
}

#[derive(Debug)]
pub struct ActiveGame {
    pub chess: Chess,
    pub engine: Arc<Mutex<GameEngine>>,
    pub status: GameStatus,
}

impl ActiveGame {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let engine = GameEngine::new()?;
        Ok(ActiveGame {
            chess: Chess::default(),
            engine: Arc::new(Mutex::new(engine)),
            status: GameStatus::WaitingForPlayer,
        })
    }

    pub fn disconnect(&mut self) {
        if matches!(self.status, GameStatus::Running) {
            self.status = GameStatus::Disconnected;
        }
    }

    pub fn connect(&mut self) {
        if matches!(self.status, GameStatus::Disconnected) {
            self.status = GameStatus::Running;
        }
    }

    pub fn finish(&mut self) {
        self.status = GameStatus::Finished;
    }
}

pub type GameStorage = Arc<RwLock<HashMap<String, Arc<Mutex<ActiveGame>>>>>;

#[derive(Clone)]
pub struct SessionManager {
    games: GameStorage,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            games: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn generate_session_id() -> String {
        Uuid::new_v4().to_string()
    }

    pub async fn create_game(
        &self,
        session_id: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let session_id = session_id.unwrap_or_else(Self::generate_session_id);

        let mut games = self.games.write().await;

        if games.len() >= 3 {
            return Err("Maximum number of active games (3) reached".into());
        }

        let game = ActiveGame::new()?;
        let game_arc = Arc::new(Mutex::new(game));

        games.insert(session_id.clone(), game_arc);

        Ok(session_id)
    }

    pub async fn get_game(&self, session_id: &str) -> Option<Arc<Mutex<ActiveGame>>> {
        let games = self.games.read().await;
        games.get(session_id).cloned()
    }

    pub async fn disconnect_game(&self, session_id: &str) -> bool {
        if let Some(game_arc) = self.get_game(session_id).await {
            let mut game = game_arc.lock().await;
            game.disconnect();
            true
        } else {
            false
        }
    }

    pub async fn connect_game(&self, session_id: &str) -> bool {
        if let Some(game_arc) = self.get_game(session_id).await {
            let mut game = game_arc.lock().await;
            game.connect();
            true
        } else {
            false
        }
    }

    pub async fn remove_game(&self, session_id: &str) -> bool {
        let mut games = self.games.write().await;
        if let Some(game_arc) = games.remove(session_id) {
            let mut game = game_arc.lock().await;
            game.finish();
            true
        } else {
            false
        }
    }
}
