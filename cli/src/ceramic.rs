use std::io::BufRead;
use std::path::Path;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Command;

fn log_errors(stdout: Vec<u8>) {
    let out = std::io::Cursor::new(stdout);
    for l in std::io::BufReader::new(out).lines() {
        if let Ok(l) = l {
            log::error!("{}", l);
        }
    }
}

pub async fn install_ceramic(name: &str, working_directory: &Path) -> anyhow::Result<()> {
    log::info!("Checking for npx");
    if !Command::new("command")
        .args(&["-v", "npx"])
        .status()
        .await?
        .success()
    {
        anyhow::bail!("npx was not found, please install node.js")
    }

    log::info!("Cloning create-ceramic-app");
    let mut child = Command::new("npx")
        .args(&["@ceramicnetwork/create-ceramic-app", "clone"])
        .current_dir(&working_directory)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("No stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("No stdout"))?;

    let mut lines = tokio::io::BufReader::new(stdout).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        log::warn!("Received {}", line);
        if line.contains("Ok to proceed") {
            stdin.write_all("y\n".as_bytes()).await?;
        }
    }
    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("Failed to clone ceramic app");
    }

    let ceramic_dir = working_directory.join("create-ceramic-app");

    log::info!("Installing required ceramic dependencies");
    let out = Command::new("npm")
        .args(&["i"])
        .current_dir(&ceramic_dir)
        .output()
        .await?;
    if !out.status.success() {
        log_errors(out.stdout);
        anyhow::bail!("Failed to install ceramic dependencies");
    }

    log::info!("Starting ceramic as a daemon");
    let out = Command::new("npx")
        .args(&[
            "@ceramicnetworkcli/daemon",
            "daemon",
            "--network",
            "inmemory",
        ]) //TODO: Check on this command flag
        .current_dir(&ceramic_dir)
        .output()
        .await?;
    if !out.status.success() {
        log_errors(out.stdout);
        anyhow::bail!("Failed to start ceramic daemon");
    }

    Ok(())
}
