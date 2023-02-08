use std::path::Path;

use tokio::process::Command;

pub async fn install_compose_db(
    _cfg: &ceramic_config::Config,
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

    log::info!("Installing composedb cli");
    let mut program = "@composedb/cli".to_string();
    if let Some(v) = version.as_ref() {
        program.push_str(&format!("@{}", v.to_string()));
    }
    let status = Command::new("npm")
        .args(&["install", &program])
        .current_dir(working_directory)
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("Failed to install composedb cli");
    }

    //TODO: need to wrap this to talk to correct host

    log::info!("ComposeDB cli now available");

    Ok(())
}
