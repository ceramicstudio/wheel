use inquire::*;
use std::path::Path;
use std::process::Stdio;

use crate::install::log_async_errors;
use crate::install::verify_db;
use ceramic_config::Config;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn install_ceramic_daemon(
    working_directory: &Path,
    cfg: &Config,
    version: &Option<semver::Version>,
    with_ceramic: bool,
) -> anyhow::Result<()> {
    verify_db::verify(&cfg).await?;

    if let Some(file_logger) = &cfg.logger.file {
        if file_logger.enabled && !file_logger.directory.exists() {
            let path_to_create = if file_logger.directory.is_absolute() {
                file_logger.directory.clone()
            } else {
                working_directory.join(&file_logger.directory)
            };
            tokio::fs::create_dir_all(path_to_create).await?;
        }
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

    let cfg_file_path = working_directory.join("daemon_config.json");
    let daemon_config: ceramic_config::DaemonConfig = cfg.clone().into();
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open(&cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string(&daemon_config)?.as_bytes())
        .await?;
    f.flush().await?;

    let ans = Confirm::new(&format!("Would you like ceramic started as a daemon?"))
        .with_default(true)
        .prompt()?;

    if ans {
        log::info!("Starting ceramic as a daemon");
        let mut cmd = Command::new("npx");
        if with_ceramic {
            cmd.env("CERAMIC_ENABLE_EXPERIMENTAL_COMPOSE_DB", "true");
        }

        let mut process = cmd
            .args(&[
                "ceramic",
                "daemon",
                "--config",
                cfg_file_path.to_string_lossy().as_ref(),
            ])
            .current_dir(working_directory)
            .kill_on_drop(false)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        tokio::spawn(async move {
            let err = process.stderr.take();
            if let Ok(exit) = process.wait().await {
                log::info!(
                    "Ceramic exited with code {}",
                    exit.code().unwrap_or_else(|| 0)
                );
                if !exit.success() {
                    if let Some(err) = err {
                        log_async_errors(err).await;
                    }
                }
            }
        });
    } else {
        log::info!("When you would like to run ceramic please run `CERAMIC_ENABLE_EXPERIMENTAL_COMPOSE_DB=true npx ceramic daemon`");
    }

    Ok(())
}
