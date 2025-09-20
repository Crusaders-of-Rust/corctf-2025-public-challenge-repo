use sol_ctf_framework::ChallengeBuilder;

use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program, sysvar,
};

use anchor_lang::{InstructionData, ToAccountMetas};
use solana_vote_program::{vote_instruction::{create_account_with_config, CreateVoteAccountConfig}, vote_state::VoteInit};

use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream}, time::{SystemTime, UNIX_EPOCH},
};

const ONE_SOL_IN_LAMPORTS: u64 = 1_000_000_000;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:5000")?;
    println!("crypto/corcoin listening on 0.0.0.0:5000");

    loop {
        let (mut stream, addr) = listener.accept()?;
        println!("new connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream.try_clone().unwrap()).await {
                println!("error handling connection from {}: {}", addr, e);
                let _ = writeln!(stream, "Error: {e}");
            }
        });
    }
}

async fn handle_connection(mut socket: TcpStream) -> std::result::Result<(), Box<dyn Error>> {
    writeln!(socket, "~~ corcoin ~~\n")?;

    let mut builder = ChallengeBuilder::try_from(socket.try_clone().unwrap()).unwrap();

    let player_program_pubkey = match builder.input_program() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            writeln!(socket, "Error: cannot load player program -> {e}")?;
            return Ok(());
        }
    };
    let program_pubkey = builder.add_program(&"./corcoin.so", Some(Pubkey::from_str_const("AHtNDB1hvKSNbtb3PYo1Eh8gvHuH1F1c5HcLeAvNPa8d"))).expect("Duplicate pubkey supplied");

    writeln!(socket, "Programs loaded:")?;
    writeln!(socket, "\tcorcoin: {}", program_pubkey)?;
    writeln!(socket, "\tplayer:  {}", player_program_pubkey)?;
    writeln!(socket, "")?;

    let admin = Keypair::new();
    builder.builder.add_account(
        admin.pubkey(),
        Account::new(1000 * ONE_SOL_IN_LAMPORTS, 0, &system_program::ID),
    );

    let player = Keypair::new();
    builder.builder.add_account(
        player.pubkey(),
        Account::new(400 * ONE_SOL_IN_LAMPORTS, 0, &system_program::ID),
    );

    writeln!(socket, "Player account created: {}", player.pubkey())?;
    writeln!(socket, "")?;

    let validator = Keypair::new();
    builder.builder.add_account(
        validator.pubkey(),
        Account::new(100 * ONE_SOL_IN_LAMPORTS, 0, &system_program::ID),
    );

    let validator_vote = Keypair::new();

    let mut challenge = builder.build().await;

    let rent = challenge.ctx.banks_client.get_rent().await?;
    let vote_config_rent = rent.minimum_balance(solana_vote_program::vote_state::VoteState::size_of());
    let validator_create_ixs = create_account_with_config(
        &admin.pubkey(),
        &validator_vote.pubkey(),
        &VoteInit {
            node_pubkey: validator.pubkey(),
            authorized_voter: validator.pubkey(),
            authorized_withdrawer: validator.pubkey(),
            commission: 0,
        },
        vote_config_rent,
        CreateVoteAccountConfig {
            space: solana_vote_program::vote_state::VoteState::size_of() as u64,
            with_seed: None
        }
    );

    challenge.run_ixs_full(
        &validator_create_ixs,
        &[&admin, &validator, &validator_vote],
        &admin.pubkey(),
    ).await?;

    writeln!(socket, "Validator account created:")?;
    writeln!(socket, "\tValidator: {}", validator.pubkey())?;
    writeln!(socket, "\tVote Account: {}", validator_vote.pubkey())?;
    writeln!(socket, "")?;

    let (config_pda, _) = Pubkey::find_program_address(&[b"config"], &program_pubkey);
    let (gcp_pda, _) = Pubkey::find_program_address(&[b"gcp"], &program_pubkey);
    let initialize_ix = solana_sdk::instruction::Instruction {
        program_id: program_pubkey,
        accounts: corcoin::accounts::Initialize {
            initializer: admin.pubkey(),
            config: config_pda,
            gcp: gcp_pda,
            system_program: system_program::ID,
        }.to_account_metas(None),
        data: corcoin::instruction::Initialize {
            admin: admin.pubkey(),
            amount: 800 * ONE_SOL_IN_LAMPORTS
        }
        .data(),
    };

    challenge.run_ixs_full(
        &[initialize_ix],
        &[&admin],
        &admin.pubkey(),
    ).await?;

    writeln!(socket, "corcoin program initialized...")?;

    let mut reader = BufReader::new(socket.try_clone()?);
    loop {
        writeln!(socket, "Choose an option:")?;
        writeln!(socket, "1) Run player program")?;
        writeln!(socket, "2) Advance clock")?;
        writeln!(socket, "3) Get flag")?;
        writeln!(socket, "4) Check player lamports")?;
        writeln!(socket, "5) Exit")?;
        write!(socket, "Choose an option [1-5]:")?;

        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;

        match buffer.trim() {
            "1" => {
                writeln!(socket, "Running player program...")?;
                let ix = challenge.read_instruction(player_program_pubkey)?;

                challenge.run_ixs_full(
                    &[ix],
                    &[&player],
                    &player.pubkey(),
                ).await.unwrap();
            }
            "2" => {
                writeln!(socket, "Advancing clock...")?;
                let clock_sysvar = challenge.ctx.banks_client.get_sysvar::<sysvar::clock::Clock>().await?;
                let mut new_clock = clock_sysvar.clone();
                challenge.ctx.warp_forward_force_reward_interval_end()?;
                new_clock.unix_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
                challenge.ctx.set_sysvar(&new_clock);
            }
            "3" => {
                writeln!(socket, "Getting flag...")?;
                break;
            }
            "4" => {
                writeln!(socket, "Checking player lamports...")?;
                let player_balance = challenge.ctx.banks_client.get_balance(player.pubkey()).await?;
                writeln!(socket, "Player lamports: {}", player_balance)?;
            }
            "5" => {
                writeln!(socket, "Exiting...")?;
                return Ok(());
            }
            _ => {
                writeln!(socket, "Invalid option")?;
            }
        }
    }

    writeln!(socket, "")?;

    let player_balance = challenge.ctx.banks_client.get_balance(player.pubkey()).await?;
    if player_balance >= 500 * ONE_SOL_IN_LAMPORTS {
        let flag = std::env::var("FLAG").unwrap_or("corctf{test_flag}".to_string());
        writeln!(socket, "Flag: {}", flag)?;
    } else {
        writeln!(socket, "Nice try...")?;
    }

    Ok(())
}