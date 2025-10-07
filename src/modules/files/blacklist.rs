use console::Emoji;
use itertools::Itertools;
use std::io::Read;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
    time::Duration,
};
use tokio::time::sleep;

use crate::{TOKEN_BLACKLIST, WALLET_BLACKLIST, log};
pub async fn watch_wallet_blacklist_file(file_path: PathBuf) {
    // Initial full load:
    let mut file = File::open(&file_path).expect("Blacklist file open failed");
    let mut reader = BufReader::new(&file);
    let mut blacklist = Vec::new();

    for line in reader.by_ref().lines() {
        let l = line.expect("Failed to read line");
        blacklist.push(l);
    }

    {
        let mut wl = WALLET_BLACKLIST.write().await;
        *wl = blacklist.into_iter().unique().collect();
    }

    // Track where we read until
    let mut position = file.seek(SeekFrom::Current(0)).unwrap();

    loop {
        sleep(Duration::from_secs(5)).await; // check interval

        // Reopen and seek to last position
        let mut file = File::open(&file_path).expect("Blacklist file open failed");
        file.seek(SeekFrom::Start(position)).expect("Seek failed");
        let mut reader = BufReader::new(&file);

        let mut new_lines = Vec::new();
        for line in reader.by_ref().lines() {
            let l = line.expect("Failed to read line");
            new_lines.push(l);
        }

        if !new_lines.is_empty() {
            let mut wl = WALLET_BLACKLIST.write().await;
            wl.extend(new_lines.into_iter());
            wl.sort_unstable();
            wl.dedup();
        }

        // Update position tracker
        position = file.seek(SeekFrom::Current(0)).unwrap()
    }
}

pub async fn watch_token_blacklist_file(file_path: PathBuf) {
    // Initial full load:
    let mut file = File::open(&file_path).expect("Blacklist file open failed");
    let mut reader = BufReader::new(&file);
    let mut blacklist = Vec::new();

    for line in reader.by_ref().lines() {
        let l = line.expect("Failed to read line");
        blacklist.push(l);
    }

    {
        let mut wl = TOKEN_BLACKLIST.write().await;
        *wl = blacklist.into_iter().unique().collect();
    }

    // Track where we read until
    let mut position = file.seek(SeekFrom::Current(0)).unwrap();

    loop {
        sleep(Duration::from_secs(5)).await; // check interval

        // Reopen and seek to last position
        let mut file = File::open(&file_path).expect("Blacklist file open failed");
        file.seek(SeekFrom::Start(position)).expect("Seek failed");
        let mut reader = BufReader::new(&file);

        let mut new_lines = Vec::new();
        for line in reader.by_ref().lines() {
            let l = line.expect("Failed to read line");
            new_lines.push(l);
        }

        if !new_lines.is_empty() {
            let mut wl = TOKEN_BLACKLIST.write().await;
            wl.extend(new_lines.into_iter());
            wl.sort_unstable();
            wl.dedup();
        }

        // Update position tracker
        position = file.seek(SeekFrom::Current(0)).unwrap();
    }
}

pub async fn show_blacklist_length() {
    let wallet_blacklist = WALLET_BLACKLIST.read().await;
    let token_blacklist = TOKEN_BLACKLIST.read().await;
    log!(
        "\t[ {} Loaded ]\t\t{} blacked wallets\t\t{} blacked tokens.",
        Emoji("💳", ""),
        wallet_blacklist.len(),
        token_blacklist.len()
    );
    sleep(Duration::from_millis(10000)).await;
}
