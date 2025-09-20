#![allow(dead_code)]

mod app;
mod db;

use std::panic::AssertUnwindSafe;

use anyhow::Result;
use futures::FutureExt;

#[tokio::main]
async fn main() -> Result<()> {
    let channels = db::get_channels().await?;
    println!("Channels:");
    println!("{}", "-".repeat(10));
    for channel in channels {
        println!("- {}", channel.name());
    }

    let fut = AssertUnwindSafe(app::App::run("ctf".to_string()));
    match fut.catch_unwind().await {
        Ok(Ok(())) => (),
        Ok(Err(e)) => eprintln!("App exited with error: `{e}`"),
        Err(_) => {
            println!("App exited with critical error - dumping info for debugging");
            println!("Please don't leak any sensitive info :(");
            println!(
                "{}",
                std::fs::read_to_string("/root/flag.txt")
                    .unwrap_or_else(|_| "corctf{example_flag}".to_string())
            );
        }
    }
    Ok(())
}
