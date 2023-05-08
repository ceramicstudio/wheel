use ceramic_config::convert_network_identifier;
use std::path::Path;
use tokio::io::AsyncWriteExt;

use crate::did::DidAndPrivateKey;
use crate::install::npm::npm_install;

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
    npm_install(working_directory, &program).await?;

    let hostname = format!("http://{}:{}", cfg.http_api.hostname, cfg.http_api.port);
    let env_file = working_directory.join("composedb.env");
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open(&env_file)
        .await?;

    f.write_all(format!("export DID_PRIVATE_KEY={}", admin_did.pk()).as_bytes())
        .await?;
    f.write_all(format!("\nexport CERAMIC_URL={}", hostname).as_bytes())
        .await?;
    f.flush().await?;

    crate::install::create_invoke_script(
        working_directory
            .join("node_modules")
            .join(".bin")
            .join("composedb"),
        working_directory.join("composedb"),
        &format!("source {}", env_file.display()),
    )
    .await?;

    log::info!(
        r#"ComposeDB cli now available.

You can run composedb with

    ./composedb

To list available models for usage, use

    ./composedb model:list --network={} --table

To run the graphiql server use

    ./composedb graphql:server --graphiql --port 5005 <path to compiled composite>
    
For more information on composedb and commands to run, see https://composedb.js.org/docs/0.4.x/first-composite

You can also take a look at https://github.com/ceramicstudio/EthDenver2023Demo for more ideas on using ComposeDB.
        "#,
        convert_network_identifier(&cfg.network.id)
    );

    Ok(())
}
