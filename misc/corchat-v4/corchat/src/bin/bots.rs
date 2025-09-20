#![allow(dead_code)]

#[path = "../db.rs"]
mod db;

use std::{
    collections::{HashMap, hash_map::Entry},
    time::Duration,
};

use anyhow::{Result, bail};

fn get_messages() -> Result<Vec<(String, String, String)>> {
    let Some(path) = std::env::args().nth(1) else {
        bail!("no path");
    };
    let messages = std::fs::read_to_string(path)?;
    Ok(messages
        .lines()
        .filter_map(|line| {
            let parts = line.split('|').collect::<Vec<_>>();
            if parts.len() == 3 {
                Some((
                    parts[0].to_owned(),
                    parts[1].to_owned(),
                    parts[2].to_owned(),
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>())
}

#[tokio::main]
async fn main() -> Result<()> {
    db::drop().await?;
    // Content not needed for challenge
    let messages = get_messages().unwrap();
    let channels = {
        let mut map = HashMap::new();
        for (_, name, _) in &messages {
            match map.entry(name.clone()) {
                Entry::Occupied(_) => (),
                Entry::Vacant(v) => {
                    v.insert(db::add_channel(name).await.unwrap());
                }
            };
        }
        map
    };

    for (user, channel, content) in messages {
        tokio::time::sleep(Duration::from_secs(5)).await;
        channels[&channel].send_message(&user, &content).await?
    }
    Ok(())
}
