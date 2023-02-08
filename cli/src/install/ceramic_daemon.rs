use crate::install::log_errors;

use std::path::Path;

use tokio::process::Command;

pub async fn install_ceramic_daemon(
    working_directory: &Path,
    version: &Option<semver::Version>,
) -> anyhow::Result<()> {
    log::info!("Checking for npm");
    if !Command::new("command")
        .args(&["-v", "npm"])
        .status()
        .await?
        .success()
    {
        anyhow::bail!("npx was not found, please install node.js")
    }

    log::info!("Installing ceramic cli");
    let mut program = "@ceramicnetwork/cli".to_string();
    if let Some(v) = version.as_ref() {
        program.push_str(&format!("@{}", v.to_string()));
    }
    let status = Command::new("npm")
        .args(&["install", &program])
        .current_dir(working_directory)
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("Failed to install ceramic cli");
    }

    log::info!("Starting ceramic as a daemon");
    let out = Command::new("npx")
        .args(&["ceramic", "daemon"])
        .current_dir(working_directory)
        .output()
        .await?;
    if !out.status.success() {
        log_errors(out.stdout);
        anyhow::bail!("Failed to start ceramic daemon");
    }

    Ok(())
}
