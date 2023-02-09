use crate::install::log_errors;

use std::path::Path;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Command;

pub async fn install_ceramic_app_template(working_directory: &Path) -> anyhow::Result<()> {
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

    Ok(())
}
