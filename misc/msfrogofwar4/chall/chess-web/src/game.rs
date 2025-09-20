use serde::{Deserialize, Serialize};
use shakmaty::{fen::Fen, CastlingMode, Chess, Color, Position};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub pos: String,
    pub moves: Vec<String>,
    pub turn_counter: u32,
    pub status: String,
    pub flag: Option<String>,
}

pub fn create_game_state(chess: &Chess) -> GameState {
    let (status, flag) = generate_game_status(chess);
    let moves = get_legal_moves(chess);
    let fen = Fen::from_position(chess, shakmaty::EnPassantMode::Legal);

    GameState {
        pos: fen.to_string(),
        moves,
        turn_counter: chess.fullmoves().into(),
        status,
        flag,
    }
}

pub fn get_legal_moves(chess: &Chess) -> Vec<String> {
    chess
        .legal_moves()
        .iter()
        .map(|s| s.to_uci(CastlingMode::Standard).to_string())
        .collect()
}

fn generate_game_status(chess: &Chess) -> (String, Option<String>) {
    if chess.is_checkmate() {
        let winner = !chess.turn();

        if winner == Color::White {
            let flag = std::env::var("FLAG").unwrap_or_else(|_| "corctf{test_flag}".to_string());
            return ("win".to_string(), Some(flag));
        } else {
            return ("lose".to_string(), None);
        }
    }

    if chess.is_stalemate() || chess.is_insufficient_material() {
        return ("draw".to_string(), None);
    }

    ("running".to_string(), None)
}
