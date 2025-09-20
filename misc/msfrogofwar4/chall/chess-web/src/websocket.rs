use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use rand::seq::SliceRandom;
use shakmaty::{uci::UciMove, Chess, Color, Position};
use std::str::FromStr;
use std::sync::Arc;

use crate::engine::{GameEngine, MoveFailureReason};
use crate::game::{create_game_state, get_legal_moves};
use crate::session::{GameStatus, SessionManager};

#[derive(serde::Serialize, serde::Deserialize)]
struct SessionMessage {
    session_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SessionResponse {
    session_id: String,
    game_found: bool,
}

pub async fn handle_socket(socket: WebSocket, session_manager: Arc<SessionManager>) {
    let (mut sender, mut receiver) = socket.split();

    send_chat_message(&mut sender, "System", "Send WASM to start new game")
        .await
        .ok();

    let mut session_id = None;

    loop {
        let msg = match receiver.next().await {
            Some(Ok(msg)) => msg,
            _ => return,
        };

        match msg {
            Message::Text(text) => {
                if let Ok(session_msg) = serde_json::from_str::<SessionMessage>(&text) {
                    if let Some(sid) = session_msg.session_id {
                        if let Some(result) =
                            handle_session_reconnect(&mut sender, &session_manager, &sid).await
                        {
                            session_id = Some(sid);
                            if result {
                                break;
                            }
                        }
                    }
                }
            }
            Message::Binary(data) => {
                println!("Received WASM binary data: {} bytes", data.len());

                if let Some(ref sid) = session_id {
                    if let Some(existing_game) = session_manager.get_game(sid).await {
                        let game = existing_game.lock().await;
                        if !matches!(game.status, GameStatus::WaitingForPlayer) {
                            send_chat_message(
                                &mut sender,
                                "System",
                                "Game already has a player WASM loaded",
                            )
                            .await
                            .ok();
                            continue;
                        }
                    }
                }

                let new_session_id = match session_manager.create_game(session_id.clone()).await {
                    Ok(sid) => sid,
                    Err(e) => {
                        send_chat_message(
                            &mut sender,
                            "System",
                            &format!("Failed to create game: {}", e),
                        )
                        .await
                        .ok();
                        return;
                    }
                };

                session_id = Some(new_session_id.clone());
                let game_arc = session_manager.get_game(&new_session_id).await;

                send_session_response(&mut sender, &new_session_id, true)
                    .await
                    .ok();
                send_chat_message(
                    &mut sender,
                    "System",
                    &format!("Received WASM file ({} bytes), loading...", data.len()),
                )
                .await
                .ok();

                if let Some(game_arc) = game_arc {
                    if load_engines(&game_arc, &data, &mut sender).await {
                        println!("New game started with session: {}", new_session_id);
                        break;
                    } else {
                        return;
                    }
                }
            }
            _ => return,
        }
    }

    let Some(ref sid) = session_id else {
        return;
    };
    let Some(game_arc) = session_manager.get_game(sid).await else {
        return;
    };

    run_game_loop(&mut sender, &mut receiver, &session_manager, sid, &game_arc).await;
}

async fn handle_session_reconnect(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    session_manager: &SessionManager,
    sid: &str,
) -> Option<bool> {
    let existing_game = session_manager.get_game(sid).await?;
    let game = existing_game.lock().await;
    let status = game.status.clone();
    drop(game);

    match status {
        GameStatus::WaitingForPlayer => {
            send_session_response(sender, sid, true).await.ok()?;
            send_chat_message(
                sender,
                "System",
                "Reconnected! Please upload your WASM file",
            )
            .await
            .ok()?;
            Some(false)
        }
        GameStatus::Disconnected | GameStatus::Running => {
            println!("Client reconnected to game: {}", sid);
            send_session_response(sender, sid, true).await.ok()?;
            send_chat_message(sender, "System", "Reconnected to existing game!")
                .await
                .ok()?;
            session_manager.connect_game(sid).await;

            let game = existing_game.lock().await;
            send_game_state(sender, &game.chess).await.ok()?;
            Some(true)
        }
        GameStatus::Finished => {
            send_chat_message(
                sender,
                "System",
                "Game already finished. Send new WASM to start new game",
            )
            .await
            .ok()?;
            session_manager.remove_game(sid).await;
            None
        }
    }
}

async fn load_engines(
    game_arc: &Arc<tokio::sync::Mutex<crate::session::ActiveGame>>,
    data: &[u8],
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> bool {
    let engine_arc = {
        let game = game_arc.lock().await;
        game.engine.clone()
    };

    let mut engine = engine_arc.lock().await;

    send_chat_message(sender, "System", "Loading chess engine...")
        .await
        .ok();
    if let Err(e) = engine.load_chess_engine().await {
        send_chat_message(
            sender,
            "System",
            &format!("Failed to load chess engine: {}", e),
        )
        .await
        .ok();
        return false;
    }

    send_chat_message(sender, "System", "Loading player WASM...")
        .await
        .ok();
    if let Err(e) = engine.load_player_engine(data).await {
        send_chat_message(
            sender,
            "System",
            &format!("Failed to load player WASM: {}", e),
        )
        .await
        .ok();
        return false;
    }

    send_chat_message(sender, "System", "Engines loaded successfully!")
        .await
        .ok();

    drop(engine);

    let mut game = game_arc.lock().await;
    game.status = GameStatus::Running;
    send_game_state(sender, &game.chess).await.is_ok()
}

async fn run_game_loop(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    receiver: &mut futures_util::stream::SplitStream<WebSocket>,
    session_manager: &SessionManager,
    sid: &str,
    game_arc: &Arc<tokio::sync::Mutex<crate::session::ActiveGame>>,
) {
    loop {
        tokio::select! {
            msg = receiver.next() => {
                if msg.is_none() || matches!(msg, Some(Ok(Message::Close(_))) | Some(Err(_))) {
                    session_manager.disconnect_game(sid).await;
                    break;
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000)) => {
                let mut game = game_arc.lock().await;

                if !matches!(game.status, GameStatus::Running) {
                    break;
                }

                if game.chess.is_game_over() {
                    send_chat_message(sender, "System", "Game is over").await.ok();
                    game.status = GameStatus::Finished;
                    break;
                }

                if game.chess.turn() == Color::White {
                    let engine_arc = game.engine.clone();
                    drop(game);

                    let mut engine = engine_arc.lock().await;
                    let mut game = game_arc.lock().await;
                    if let Err(e) = process_player_turn(&mut game.chess, &mut engine, sender).await {
                        send_chat_message(sender, "System", &format!("Error: {}", e)).await.ok();
                    }
                }
            }
        }
    }

    if let Some(game_arc) = session_manager.get_game(sid).await {
        let mut game = game_arc.lock().await;
        if game.chess.is_game_over() || matches!(game.status, GameStatus::Finished) {
            game.status = GameStatus::Finished;
            drop(game);
            session_manager.remove_game(sid).await;
            println!("Game finished and removed: {}", sid);
        } else {
            println!("Game disconnected: {}", sid);
        }
    }
}

async fn process_player_turn(
    chess: &mut Chess,
    engine: &mut GameEngine,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> Result<(), String> {
    let legal_moves = get_legal_moves(chess);
    if legal_moves.is_empty() {
        send_chat_message(sender, "System", "No legal moves available")
            .await
            .ok();
        return Ok(());
    }

    let fen_string =
        shakmaty::fen::Fen::from_position(chess, shakmaty::EnPassantMode::Legal).to_string();
    let move_result = match engine.get_player_move(&fen_string, &legal_moves).await {
        Ok(result) => result,
        Err(e) => {
            send_chat_message(sender, "System", &format!("Player engine error: {}", e))
                .await
                .ok();
            return Err(e.to_string());
        }
    };

    let player_move = match move_result.move_str {
        Some(move_str) => move_str,
        None => {
            let failure_msg = match move_result.failure_reason {
                Some(MoveFailureReason::WasmPanic) => format!(
                    "Player WASM panicked after {:.0}ms, picking random move",
                    move_result.duration.as_millis()
                ),
                Some(MoveFailureReason::WasmError(ref err)) => format!(
                    "Player WASM error after {:.0}ms: {}, picking random move",
                    move_result.duration.as_millis(),
                    err
                ),
                None => "Player engine returned no move, picking random".to_string(),
            };
            send_chat_message(sender, "System", &failure_msg).await.ok();
            get_random_move(&legal_moves).unwrap_or_default()
        }
    };

    if player_move.is_empty() {
        return Err("No moves available".to_string());
    }

    let player_msg = if move_result.failure_reason.is_none() {
        format!(
            "Played move: {} ({:.0}ms, {} fuel used)",
            player_move,
            move_result.duration.as_millis(),
            move_result.fuel_used
        )
    } else {
        format!("Played move: {} (random)", player_move)
    };
    send_chat_message(sender, "Player", &player_msg).await.ok();

    execute_move(chess, &player_move)?;
    send_state_update(sender, &create_game_state(chess))
        .await
        .ok();

    if chess.fullmoves().get() >= 50 {
        send_chat_message(sender, "System", "Game ended in draw after 50 moves")
            .await
            .ok();
        return Ok(());
    }

    if chess.is_game_over() || chess.turn() != Color::Black {
        return Ok(());
    }

    process_computer_turn(chess, engine, sender).await
}

async fn process_computer_turn(
    chess: &mut Chess,
    engine: &mut GameEngine,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> Result<(), String> {
    let fen_string =
        shakmaty::fen::Fen::from_position(chess, shakmaty::EnPassantMode::Legal).to_string();
    let legal_moves = get_legal_moves(chess);
    let computer_move_result = match engine.get_computer_move(&fen_string, &legal_moves).await {
        Ok(result) => result,
        Err(e) => {
            send_chat_message(sender, "System", &format!("Computer engine error: {}", e))
                .await
                .ok();
            return Err(e.to_string());
        }
    };

    let computer_move = match computer_move_result.move_str {
        Some(move_str) => {
            send_chat_message(
                sender,
                "Computer",
                &format!(
                    "Played move: {} ({:.0}ms, {} fuel used)",
                    move_str,
                    computer_move_result.duration.as_millis(),
                    computer_move_result.fuel_used
                ),
            )
            .await
            .ok();
            move_str
        }
        None => {
            let failure_msg = match computer_move_result.failure_reason {
                Some(MoveFailureReason::WasmPanic) => format!(
                    "Computer WASM panicked after {:.0}ms, picking random move",
                    computer_move_result.duration.as_millis()
                ),
                Some(MoveFailureReason::WasmError(ref err)) => format!(
                    "Computer WASM error after {:.0}ms: {}, picking random move",
                    computer_move_result.duration.as_millis(),
                    err
                ),
                None => "Computer engine returned no move, picking random".to_string(),
            };

            let random_move = get_random_move(&legal_moves).unwrap_or_default();
            if !random_move.is_empty() {
                send_chat_message(sender, "System", &failure_msg).await.ok();
                send_chat_message(
                    sender,
                    "Computer",
                    &format!("Played move: {} (random)", random_move),
                )
                .await
                .ok();
            }
            random_move
        }
    };

    if !computer_move.is_empty() {
        execute_move(chess, &computer_move)?;
        send_state_update(sender, &create_game_state(chess))
            .await
            .ok();

        if chess.fullmoves().get() >= 50 {
            send_chat_message(sender, "System", "Game ended in draw after 50 moves")
                .await
                .ok();
        }
    }

    Ok(())
}

fn execute_move(chess: &mut Chess, move_str: &str) -> Result<(), String> {
    let uci =
        UciMove::from_str(move_str).map_err(|_| format!("Invalid move notation: {}", move_str))?;
    let chess_move = uci
        .to_move(chess)
        .map_err(|_| format!("Invalid move: {}", move_str))?;
    *chess = chess.clone().play(chess_move).unwrap();
    Ok(())
}

fn get_random_move(legal_moves: &[String]) -> Option<String> {
    if legal_moves.is_empty() {
        return None;
    }
    let mut rng = rand::thread_rng();
    legal_moves.choose(&mut rng).cloned()
}

async fn send_game_state(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    chess: &Chess,
) -> Result<(), axum::Error> {
    let state = create_game_state(chess);
    let response = serde_json::json!({
        "type": "state",
        "data": state
    });
    sender
        .send(Message::Text(response.to_string().into()))
        .await
}

async fn send_state_update(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    state: &crate::game::GameState,
) -> Result<(), axum::Error> {
    let response = serde_json::json!({
        "type": "state",
        "data": state
    });
    sender
        .send(Message::Text(response.to_string().into()))
        .await
}

async fn send_chat_message(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    name: &str,
    message: &str,
) -> Result<(), axum::Error> {
    let response = serde_json::json!({
        "type": "chat",
        "data": {
            "name": name,
            "msg": message
        }
    });
    sender
        .send(Message::Text(response.to_string().into()))
        .await
}

async fn send_session_response(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    session_id: &str,
    game_found: bool,
) -> Result<(), axum::Error> {
    let response = SessionResponse {
        session_id: session_id.to_string(),
        game_found,
    };
    sender
        .send(Message::Text(
            serde_json::to_string(&response).unwrap().into(),
        ))
        .await
}
