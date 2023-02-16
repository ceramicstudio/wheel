use ssi::did::Document;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn install_compose_db(
    cfg: &ceramic_config::Config,
    admin_did: &Document,
    working_directory: &Path,
    version: &Option<semver::Version>,
) -> anyhow::Result<()> {
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

    let hostname = format!("http://{}:{}", cfg.http_api.hostname, cfg.http_api.port);
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open(working_directory.join("composedb.env"))
        .await?;

    f.write_all(format!("export DID_PRIVATE_KEY={}", admin_did.id.to_string()).as_bytes())
        .await?;
    f.write_all(format!("export CERAMIC_URL={}", hostname).as_bytes())
        .await?;
    f.flush().await?;

    log::info!(
        r#"ComposeDB cli now available. To properly use composedb, you will need to update your environment

    cd ${}
    source composedb.env

You can then run composedb with

    $(npm bin)/composedb"#,
        working_directory.to_string_lossy()
    );

    Ok(())
}
