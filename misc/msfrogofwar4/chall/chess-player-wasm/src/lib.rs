wit_bindgen::generate!();

use rand::seq::IndexedRandom;
use crate::exports::corctf::player::player::Guest;
struct ChessEngineImpl;

impl Guest for ChessEngineImpl {
    fn init() {

    }

    fn play(_fen: String, moves: Vec<String>) -> String {
        let mut rng = rand::rng();
        let chosen_move = moves.choose(&mut rng).unwrap();
        chosen_move.to_string()
    }
}

export!(ChessEngineImpl);
