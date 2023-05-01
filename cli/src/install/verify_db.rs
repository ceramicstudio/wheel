use ceramic_config::{convert_network_identifier, Config};
use sqlx::{Connection, Executor, Row};
use std::path::PathBuf;

const SELECT_NETWORK_OPTION: &'static str =
    "SELECT value FROM ceramic_config WHERE option = 'network'";
const VALUE_INDEX: &'static str = "value";

pub async fn verify(cfg: &Config) -> anyhow::Result<()> {
    log::info!(
        "Verifying database connection using connection string {}",
        cfg.indexing.db
    );
    if cfg.indexing.db.starts_with("postgres") {
        match sqlx::postgres::PgConnection::connect(&cfg.indexing.db).await {
            Err(e) => {
                log::error!(
                    "Failed to connect to postgres, aborting daemon startup\n    {}",
                    e
                );
                log::error!("For more information on setting up postgres see https://github.com/3box/wheel#setting-up-postgres");
                return Err(e.into());
            }
            Ok(mut c) => {
                if let Some(res) = c.fetch_optional(SELECT_NETWORK_OPTION).await? {
                    let db_network: String = res.get(VALUE_INDEX);
                    let network = convert_network_identifier(&cfg.network.id);
                    if network != db_network {
                        let err = anyhow::anyhow!(
                            r#"Network {} ({}) does not match existing network {}.

If you want to switch  networks, please follow the removal instructions at
https://blog.ceramic.network/composedb-beta-update-model-versioning-release/ and
then recreate following https://github.com/3box/wheel#setting-up-postgres"#,
                            cfg.network.id,
                            network,
                            db_network
                        );
                        return Err(err);
                    }
                }
            }
        }
    } else {
        let (_, path) = cfg.indexing.db.split_once("://").unwrap();
        let p = PathBuf::from(path);
        log::info!("Verifying sqlite path exists at {}", p.display());
        if tokio::fs::try_exists(p).await? {
            if let Ok(mut c) = sqlx::sqlite::SqliteConnection::connect(&cfg.indexing.db).await {
                if let Some(res) = c.fetch_optional(SELECT_NETWORK_OPTION).await? {
                    let db_network: String = res.get(VALUE_INDEX);
                    let network = convert_network_identifier(&cfg.network.id);
                    if network != db_network {
                        let err = anyhow::anyhow!(
                            r#"Network {} ({}) does not match existing network {}.

If you want to switch networks, please follow the removal instructions at
https://blog.ceramic.network/composedb-beta-update-model-versioning-release/."#,
                            cfg.network.id,
                            network,
                            db_network
                        );
                        return Err(err);
                    }
                }
            }
        } else {
            let err = anyhow::anyhow!("Cannot connect sqlite at path {}, aborting startup", path);
            log::error!("{}", err.to_string());
            return Err(err);
        }
    }
    Ok(())
}
