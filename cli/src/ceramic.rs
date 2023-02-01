use std::io::BufRead;
use std::path::PathBuf;

use tokio::process::Command;

use tokio::process::{Command, ChildStdout};

fn log_errors(stdout: Vec<u8>) {
    for l in std::io::BufReader::from(stdout).lines() {
        if let Ok(l) = l {
            log::error!(l);
        }
    }
}

pub async fn install_ceramic(working_directory: PathBuf) -> anyhow::Result<()> {
    if !Command::new("command")
        .args(&["-v", "npx"])
        .status().await?
        .success() {
        anyhow::bail!("npx was not found, please install node.js")
    }

    log::info!("Cloning create-ceramic-app");
    let out = Command::new("npx")
        .args(&["@ceramicnetwork/create-ceramic-app", "clone"])
        .current_dir(working_directory.clone())
        .output().await?;
    if !out.status.success() {
        log_errors(out.stdout);
        anyhow::bail!("Failed to clone ceramic app");
    }

    let ceramic_dir = working_directory.join("create-ceramic-app");

    log::info!("Installing required ceramic dependencies");
    let out = Command::new("npm")
        .args(&["i"])
        .current_dir(ceramic_dir)
        .output().await?;
    if !out.status.success() {
        log_errors(out.stdout);
        anyhow::bail!("Failed to install ceramic dependencies");
    }

    log::info!("Starting ceramic as a daemon");
    let out = Command::new("npx")
        .args(&["@ceramicnetworkcli/daemon", "daemon", "--network", "inmemory"]) //TODO: Check on this command flag
        .current_dir(ceramic_dir)
        .output().await?;
    if !out.status.success() {
        log_errors(out.stdout);
        anyhow::bail!("Failed to start ceramic daemon");
    }

    Ok(())
}