use sol_ctf_framework::ChallengeBuilder;

use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program, sysvar,
};

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};

use std::{
    error::Error,
    io::Write,
    net::{TcpListener, TcpStream},
};

use rand::{RngCore, rngs::OsRng};

#[derive(Clone, Copy)]
enum Move {
    Rock = 0,
    Paper = 1,
    Scissors = 2,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:5001")?;
    println!("crypto/sol_rps listening on 0.0.0.0:5001");

    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("new connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream.try_clone().unwrap()).await {
                let _ = writeln!(stream, "Error: {e}");
            }
        });
    }
}

async fn handle_connection(mut socket: TcpStream) -> std::result::Result<(), Box<dyn Error>> {
    writeln!(socket, "~~ sol_rps ~~\n")?;

    let mut builder = ChallengeBuilder::try_from(socket.try_clone().unwrap()).unwrap();

    let player_program_pubkey = match builder.input_program() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            writeln!(socket, "Error: cannot load player program -> {e}")?;
            return Ok(());
        }
    };

    let manager_pubkey = builder
        .add_program("./manager.so", Some(Pubkey::from_str_const("GdWp89pQC5n36HAznAYxqE6ev9dGvrH2cPMn5BQjJ6nW")))
        .ok_or("Failed to load manager program")?;
    let rps_pubkey = builder
        .add_program("./rps.so", Some(Pubkey::from_str_const("63BgSDqQVJdfX9HDsxFUKJfjt45pnEPpT2CoWMwPNsho")))
        .ok_or("Failed to load rps program")?;

    writeln!(socket, "Programs loaded:")?;
    writeln!(socket, "   Manager: {}", manager_pubkey)?;
    writeln!(socket, "   RPS:     {}", rps_pubkey)?;
    writeln!(socket, "   Player:  {}", player_program_pubkey)?;
    writeln!(socket, "")?;

    let admin = Keypair::new();
    builder.builder.add_account(
        admin.pubkey(),
        Account::new(1_000_000_000, 0, &system_program::ID),
    );

    let (config_pda, _) = Pubkey::find_program_address(&[b"config"], &manager_pubkey);

    let mut challenge = builder.build().await;

    writeln!(socket, "Initializing config...")?;

    let initialize_accounts = manager::accounts::Initialize {
        config: config_pda,
        authority: admin.pubkey(),
        system_program: system_program::ID,
    };

    let initialize_ix = solana_sdk::instruction::Instruction {
        program_id: manager_pubkey,
        accounts: initialize_accounts.to_account_metas(None),
        data: manager::instruction::Initialize {
            admin: admin.pubkey(),
        }
        .data(),
    };

    challenge
        .run_ixs_full(&[initialize_ix], &[&admin], &admin.pubkey())
        .await?;

    writeln!(socket, "Config initialized...")?;

    writeln!(socket, "Generating admin moves...")?;
    let admin_moves_enum = generate_random_moves();

    let (game_pda, _) =
        Pubkey::find_program_address(&[b"game", admin.pubkey().as_ref()], &rps_pubkey);

    writeln!(socket, "Running game...")?;

    let admin_moves: [rps::Move; 100] = admin_moves_enum
        .iter()
        .map(|&m| match m {
            Move::Rock => rps::Move::Rock,
            Move::Paper => rps::Move::Paper,
            Move::Scissors => rps::Move::Scissors,
        })
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| "Failed to convert moves")?;

    let run_game_accounts = manager::accounts::RunGame {
        admin: admin.pubkey(),
        game: game_pda,
        player_program: player_program_pubkey,
        config: config_pda,
        rps_program: rps_pubkey,
        system_program: system_program::ID,
        clock: sysvar::clock::ID,
    };

    let run_game_ix = solana_sdk::instruction::Instruction {
        program_id: manager_pubkey,
        accounts: run_game_accounts.to_account_metas(None),
        data: manager::instruction::RunGame { admin_moves }.data(),
    };

    challenge
        .run_ixs_full(&[run_game_ix], &[&admin], &admin.pubkey())
        .await?;

    writeln!(socket, "Game completed!\n")?;

    let config_account = match challenge.ctx.banks_client.get_account(config_pda).await {
        Ok(Some(data)) => data,
        Ok(None) => {
            writeln!(socket, "Error: config account not found")?;
            return Ok(());
        }
        Err(e) => {
            return Err(e.into());
        }
    };

    let config = manager::Config::try_deserialize(&mut config_account.data.as_slice())?;
    writeln!(socket, "Game won: {}", config.won)?;

    if config.won {
        let flag = std::env::var("FLAG").unwrap_or_else(|_| "corctf{test_flag}".to_string());
        writeln!(socket, "Flag: {flag}")?;
    } else {
        writeln!(socket, "Better luck next time!")?;
    }

    Ok(())
}

fn generate_random_moves() -> [Move; 100] {
    let mut moves = [Move::Rock; 100];
    let mut rng = OsRng;
    let mut random_bytes = [0u8; 100];
    rng.fill_bytes(&mut random_bytes);

    for (i, &byte) in random_bytes.iter().enumerate() {
        moves[i] = match byte % 3 {
            0 => Move::Rock,
            1 => Move::Paper,
            _ => Move::Scissors,
        };
    }

    moves
}
