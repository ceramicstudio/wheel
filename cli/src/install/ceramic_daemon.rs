use inquire::*;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::task::JoinHandle;

use crate::install::log_async_errors;
use crate::install::verify_db;
use ceramic_config::Config;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const DAEMON_CONFIG: &'static str = "daemon_config.json";

pub async fn install_ceramic_daemon(
    working_directory: &Path,
    cfg: &Config,
    version: &Option<semver::Version>,
    quiet: bool,
) -> anyhow::Result<Option<JoinHandle<()>>> {
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

    let cfg_file_path = working_directory.join(DAEMON_CONFIG);
    let daemon_config: ceramic_config::DaemonConfig = cfg.clone().into();
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .append(false)
        .open(&cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string(&daemon_config)?.as_bytes())
        .await?;
    f.flush().await?;

    let ans = if quiet {
        true
    } else {
        Confirm::new(&format!("Would you like ceramic started as a daemon?"))
            .with_default(true)
            .prompt()?
    };

    let ceramic_path = PathBuf::from("node_modules").join(".bin").join("ceramic");

    let ret = if ans {
        log::info!(
            "Starting ceramic as a daemon, using config file {} and binary {}",
            cfg_file_path.display(),
            working_directory.join(&ceramic_path).display()
        );
        let mut cmd = Command::new("node");

        let mut process = cmd
            .args(&[
                ceramic_path.to_string_lossy().as_ref(),
                "daemon",
                "--config",
                DAEMON_CONFIG,
            ])
            .current_dir(working_directory)
            .kill_on_drop(false)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Some(tokio::spawn(async move {
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
        }))
    } else {
        log::info!(
            r#"When you would like to run ceramic please run 

{} daemon --config {}
            "#,
            ceramic_path.display(),
            cfg_file_path.display()
        );
        None
    };

    Ok(ret)
}
