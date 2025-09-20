use std::time::{Duration, Instant};

use anyhow::Result;
use futures_util::FutureExt;
use wasmtime::{component::*, Engine as WasmEngine, ResourceLimiterAsync, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

#[derive(Debug, Clone)]
pub enum MoveFailureReason {
    WasmPanic,
    WasmError(String),
}

#[derive(Debug, Clone)]
pub struct MoveResult {
    pub move_str: Option<String>,
    pub duration: Duration,
    pub failure_reason: Option<MoveFailureReason>,
    pub fuel_used: u64,
}

const COMPUTER_TIMEOUT: Duration = Duration::from_millis(3_000);
const COMPUTER_MAX_FUEL: u64 = 500_000_000_000;

// :)
const PLAYER_TIMEOUT: Duration = Duration::from_millis(250);
const PLAYER_MAX_FUEL: u64 = 50_000_000;

const INIT_TIMEOUT: Duration = Duration::from_millis(1000);

mod chess_engine_bindings {
    wasmtime::component::bindgen!({
        path: "../chess-engine-wasm/wit",
        exports: { default: async }
    });
}

mod chess_player_bindings {
    wasmtime::component::bindgen!({
        path: "../chess-player-wasm/wit",
        exports: { default: async }
    });
}

use chess_engine_bindings::Guest as ChessEngine;
use chess_player_bindings::Guest as ChessPlayer;

const WASM_PAGE_SIZE: usize = 65_536usize;
pub struct ComponentRunStates {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
    pub resource_limiter: ResourceLimiter,
}

pub struct ResourceLimiter {}
#[async_trait::async_trait]
impl ResourceLimiterAsync for ResourceLimiter {
    async fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool> {
        Ok(desired <= 2048 * WASM_PAGE_SIZE)
    }

    async fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool> {
        Ok(true)
    }
}

impl WasiView for ComponentRunStates {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

pub struct GameEngine {
    pub engine: WasmEngine,
    pub store: Store<ComponentRunStates>,
    pub linker: Linker<ComponentRunStates>,
    pub chess_engine: Option<ChessEngine>,
    pub player_engine: Option<ChessPlayer>,
}

impl GameEngine {
    pub fn new() -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.consume_fuel(true);
        config.async_support(true);
        let engine = WasmEngine::new(&config)?;

        let wasi = WasiCtx::builder().inherit_stdio().inherit_network().build();
        let state = ComponentRunStates {
            wasi_ctx: wasi,
            resource_table: ResourceTable::new(),
            resource_limiter: ResourceLimiter {},
        };
        let mut store = Store::new(&engine, state);
        store.fuel_async_yield_interval(Some(100_000))?;
        store.set_fuel(u64::MAX)?;
        store.limiter_async(|s| &mut s.resource_limiter);

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

        Ok(GameEngine {
            engine,
            store,
            linker,
            chess_engine: None,
            player_engine: None,
        })
    }

    pub async fn load_chess_engine(&mut self) -> Result<()> {
        let wasm_bytes = std::fs::read("./chess_engine_wasm.wasm")?;
        let component = Component::new(&self.engine, &wasm_bytes)?;
        let instance =
            ChessEngine::instantiate_async(&mut self.store, &component, &self.linker).await?;
        tokio::time::timeout(
            INIT_TIMEOUT,
            instance.corctf_engine_engine().call_init(&mut self.store),
        )
        .await??;
        self.chess_engine = Some(instance);
        Ok(())
    }

    pub async fn load_player_engine(&mut self, wasm_bytes: &[u8]) -> Result<()> {
        const MAX_WASM_SIZE: usize = 5 * 1024 * 1024; // 5MB

        if wasm_bytes.len() > MAX_WASM_SIZE {
            return Err(anyhow::anyhow!(
                "WASM binary too large: {} bytes (max: {} bytes)",
                wasm_bytes.len(),
                MAX_WASM_SIZE
            ));
        }

        let component = Component::new(&self.engine, wasm_bytes)?;
        let instance =
            ChessPlayer::instantiate_async(&mut self.store, &component, &self.linker).await?;
        tokio::time::timeout(
            INIT_TIMEOUT,
            instance.corctf_player_player().call_init(&mut self.store),
        )
        .await??;
        self.player_engine = Some(instance);
        Ok(())
    }

    pub async fn get_computer_move(
        &mut self,
        fen: &str,
        legal_moves: &[String],
    ) -> Result<MoveResult> {
        let start_time = Instant::now();
        self.store.set_fuel(COMPUTER_MAX_FUEL)?;
        let start_fuel = COMPUTER_MAX_FUEL;

        let result = std::panic::AssertUnwindSafe(async {
            if let Some(engine) = &mut self.chess_engine {
                let result = tokio::time::timeout(
                    COMPUTER_TIMEOUT,
                    engine
                        .corctf_engine_engine()
                        .call_play(&mut self.store, fen, legal_moves),
                )
                .await;

                if self.store.get_fuel()? == 0 {
                    return Ok(MoveResult {
                        move_str: None,
                        duration: start_time.elapsed(),
                        failure_reason: Some(MoveFailureReason::WasmError(
                            "Out of fuel".to_string(),
                        )),
                        fuel_used: 0,
                    });
                }

                match result {
                    Ok(Ok(move_str)) if !move_str.is_empty() => {
                        return Ok(MoveResult {
                            move_str: Some(move_str),
                            duration: start_time.elapsed(),
                            failure_reason: None,
                            fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
                        });
                    }
                    Ok(Err(e)) => {
                        return Ok(MoveResult {
                            move_str: None,
                            duration: start_time.elapsed(),
                            failure_reason: Some(MoveFailureReason::WasmError(e.to_string())),
                            fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
                        });
                    }
                    Ok(_) => {
                        return Ok(MoveResult {
                            move_str: None,
                            duration: start_time.elapsed(),
                            failure_reason: Some(MoveFailureReason::WasmError(
                                "Empty move returned".to_string(),
                            )),
                            fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
                        });
                    }
                    Err(_) => {
                        return Ok(MoveResult {
                            move_str: None,
                            duration: start_time.elapsed(),
                            failure_reason: Some(MoveFailureReason::WasmError(format!("Timeout"))),
                            fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
                        });
                    }
                }
            }
            Ok(MoveResult {
                move_str: None,
                duration: start_time.elapsed(),
                failure_reason: Some(MoveFailureReason::WasmError(
                    "Engine not loaded".to_string(),
                )),
                fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
            })
        })
        .catch_unwind()
        .await;

        match result {
            Ok(result) => result,
            Err(_) => Ok(MoveResult {
                move_str: None,
                duration: start_time.elapsed(),
                failure_reason: Some(MoveFailureReason::WasmPanic),
                fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
            }),
        }
    }

    pub async fn get_player_move(
        &mut self,
        fen: &str,
        legal_moves: &[String],
    ) -> Result<MoveResult> {
        let start_time = Instant::now();
        self.store.set_fuel(PLAYER_MAX_FUEL)?;
        let start_fuel = PLAYER_MAX_FUEL;

        let result = std::panic::AssertUnwindSafe(async {
            if let Some(player) = &mut self.player_engine {
                let result = tokio::time::timeout(
                    PLAYER_TIMEOUT,
                    player
                        .corctf_player_player()
                        .call_play(&mut self.store, fen, legal_moves),
                )
                .await;

                if self.store.get_fuel()? == 0 {
                    return Ok(MoveResult {
                        move_str: None,
                        duration: start_time.elapsed(),
                        failure_reason: Some(MoveFailureReason::WasmError(
                            "Out of fuel".to_string(),
                        )),
                        fuel_used: 0,
                    });
                }

                match result {
                    Ok(Ok(move_str)) if !move_str.is_empty() => Ok(MoveResult {
                        move_str: Some(move_str),
                        duration: start_time.elapsed(),
                        failure_reason: None,
                        fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
                    }),
                    Ok(Err(e)) => Ok(MoveResult {
                        move_str: None,
                        duration: start_time.elapsed(),
                        failure_reason: Some(MoveFailureReason::WasmError(e.to_string())),
                        fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
                    }),
                    Ok(_) => Ok(MoveResult {
                        move_str: None,
                        duration: start_time.elapsed(),
                        failure_reason: Some(MoveFailureReason::WasmError(
                            "Empty move returned".to_string(),
                        )),
                        fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
                    }),
                    Err(_) => Ok(MoveResult {
                        move_str: None,
                        duration: start_time.elapsed(),
                        failure_reason: Some(MoveFailureReason::WasmError("Timeout".to_string())),
                        fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
                    }),
                }
            } else {
                Ok(MoveResult {
                    move_str: None,
                    duration: start_time.elapsed(),
                    failure_reason: Some(MoveFailureReason::WasmError(
                        "Player engine not loaded".to_string(),
                    )),
                    fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
                })
            }
        })
        .catch_unwind()
        .await;

        match result {
            Ok(result) => result,
            Err(_) => Ok(MoveResult {
                move_str: None,
                duration: start_time.elapsed(),
                failure_reason: Some(MoveFailureReason::WasmPanic),
                fuel_used: start_fuel.saturating_sub(self.store.get_fuel()?),
            }),
        }
    }
}

impl std::fmt::Debug for GameEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameEngine")
            .field("has_chess_engine", &self.chess_engine.is_some())
            .field("has_player_engine", &self.player_engine.is_some())
            .finish()
    }
}
