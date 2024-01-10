use inquire::*;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use tokio::task::JoinHandle;

use crate::install::log_async_errors;
use crate::install::npm::npm_install_package;
use crate::install::verify_db;
use ceramic_config::Config;
use spinners::{Spinner, Spinners};
use tokio::process::Command;

enum CeramicStatus {
    Complete(Option<ExitStatus>),
    HttpComplete(reqwest::Result<reqwest::Response>),
}

pub async fn install_ceramic_daemon(
    working_directory: &Path,
    cfg: &Config,
    version: &Option<semver::Version>,
    ceramic_config_file: &Path,
    start_ceramic: Option<bool>,
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

    let mut program = "@ceramicnetwork/cli".to_string();
    if let Some(v) = version.as_ref() {
        program.push_str(&format!("@{}", v.to_string()));
    }
    npm_install_package(&working_directory, &program, false).await?;

    let ans = match start_ceramic {
        Some(true) => true,
        Some(false) => false,
        None => Confirm::new(&format!("Would you like ceramic started as a daemon?"))
            .with_default(true)
            .prompt()?,
    };

    let ceramic_path = working_directory
        .join("node_modules")
        .join(".bin")
        .join("ceramic");
    crate::install::create_invoke_script(&ceramic_path, working_directory.join("ceramic"), "")
        .await?;

    let ret = if ans {
        log::info!(
            "Starting ceramic as a daemon, using config file {} and binary {}",
            ceramic_config_file.display(),
            ceramic_path.display()
        );
        let mut cmd = Command::new("sh");

        let mut process = cmd
            .args(&[
                "ceramic",
                "daemon",
                "--config",
                &ceramic_config_file.display().to_string(),
            ])
            .current_dir(working_directory)
            .kill_on_drop(false)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let opt_child = Some(tokio::spawn(async move {
            let err = process.stderr.take();
            if let Ok(exit) = process.wait().await {
                let _ = tx.send(exit.clone()).await;
                log::info!(
                    "\nCeramic exited with code {}",
                    exit.code().unwrap_or_else(|| 0)
                );
                if !exit.success() {
                    if let Some(err) = err {
                        log_async_errors(err).await;
                    }
                }
            }
        }));

        let url = format!(
            "http://{}:{}/api/v0/node/healthcheck",
            cfg.http_api.hostname, cfg.http_api.port
        );

        let mut sp = Spinner::new(Spinners::Star2, "Waiting for ceramic to start".into());

        loop {
            let r = tokio::select! {
                r = rx.recv() => {
                    CeramicStatus::Complete(r)
                }
                r = reqwest::get(&url) => {
                    CeramicStatus::HttpComplete(r)
                }
            };
            match r {
                CeramicStatus::Complete(_) => {
                    anyhow::bail!("Ceramic failed to start");
                }
                CeramicStatus::HttpComplete(r) => {
                    match r {
                        Ok(r) => {
                            log::debug!("Ceramic responded with status {}", r.status());
                            if r.status().is_success() {
                                break;
                            }
                        }
                        Err(e) => {
                            log::debug!("Ceramic failed to respond with error {}", e);
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
        sp.stop_with_newline();
        opt_child
    } else {
        None
    };

    log::info!(
        r#"
When you would like to run ceramic please run 

    ./ceramic daemon --config {}

from the directory {}. For more information on the Ceramic http api see https://developers.ceramic.network/build/http/api/
        "#,
        ceramic_config_file.display(),
        working_directory.display()
    );

    Ok(ret)
}
