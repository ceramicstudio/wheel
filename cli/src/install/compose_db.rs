use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::did::DidAndPrivateKey;

pub async fn install_compose_db(
    cfg: &ceramic_config::Config,
    admin_did: &DidAndPrivateKey,
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

    f.write_all(format!("export DID_PRIVATE_KEY={}", admin_did.pk()).as_bytes())
        .await?;
    f.write_all(format!("\nexport CERAMIC_URL={}", hostname).as_bytes())
        .await?;
    f.flush().await?;
    let composedb_path = working_directory
        .join("node_modules")
        .join(".bin")
        .join("composedb");

    let symlink = working_directory.join("composedb");
    if !tokio::fs::try_exists(&symlink).await? {
        tokio::fs::symlink(working_directory.join(&composedb_path), symlink).await?;
    }

    log::info!(
        r#"ComposeDB cli now available. 
        
To properly use composedb, you will need to update your environment

    source {}/composedb.env

You can then run composedb with

    node composedb

To run the graphiql server use

    node composedb graphql:server --graphiql --port 5005 <path to compiled composite>
    
For more information on composedb and commands to run, see https://composedb.js.org/docs/0.4.x/first-composite
        "#,
        working_directory.display(),
    );

    Ok(())
}
