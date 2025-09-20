use std::cell::RefCell;
use timecat::prelude::*;
wit_bindgen::generate!();

thread_local! {
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
}

use crate::exports::corctf::engine::engine::Guest;
struct ChessEngineImpl;

impl Guest for ChessEngineImpl {
    fn init() {
        let board = Board::default();
        ENGINE.with(|e| {
            *e.borrow_mut() = Some(Engine::from_board(board));
        });
    }

    fn play(fen: String, _moves: Vec<String>) -> String {
        ENGINE.with(|e| {
            let Some(engine) = &mut *e.borrow_mut() else {
                return "".to_string();
            };

            if engine.set_fen(&fen).is_err() {
                return "".to_string();
            }

            let search_info = engine.search_millis(2750, false);

            search_info
                .get_best_move()
                .expect("No best move found")
                .to_string()
        })
    }
}

export!(ChessEngineImpl);
