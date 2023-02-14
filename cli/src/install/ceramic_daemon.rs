use inquire::*;
use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

pub async fn install_ceramic_daemon(
    working_directory: &Path,
    cfg_file_path: &Path,
    version: &Option<semver::Version>,
    with_ceramic: bool,
) -> anyhow::Result<()> {
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

    let ans = Confirm::new(&format!("Would you like ceramic started as a daemon?"))
        .with_default(true)
        .prompt()?;

    if ans {
        log::info!("Starting ceramic as a daemon");
        let mut cmd = Command::new("npx");
        if with_ceramic {
            cmd.env("CERAMIC_ENABLE_EXPERIMENTAL_COMPOSE_DB", "true");
        }

        cmd.args(&[
            "ceramic",
            "daemon",
            "--config",
            cfg_file_path.to_string_lossy().as_ref(),
        ])
        .current_dir(working_directory)
        .kill_on_drop(false)
        .stdout(Stdio::null())
        .spawn()?;
    } else {
        log::info!("When you would like to run ceramic please run `CERAMIC_ENABLE_EXPERIMENTAL_COMPOSE_DB=true npx ceramic daemon`");
    }

    Ok(())
}
