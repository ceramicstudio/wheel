use ceramic_config::Config;
use sqlx::Connection;
use std::path::PathBuf;

pub async fn verify(cfg: &Config) -> anyhow::Result<()> {
    log::info!(
        "Verifying database connection using connection string {}",
        cfg.indexing.db
    );
    if cfg.indexing.db.starts_with("postgres") {
        if let Err(e) = sqlx::postgres::PgConnection::connect(&cfg.indexing.db).await {
            log::error!(
                "Failed to connect to postgres, aborting daemon startup\n    {}",
                e
            );
            log::error!("For more information on setting up postgres see https://github.com/3box/wheel#setting-up-postgres");
            return Err(e.into());
        }
    } else {
        let (_, path) = cfg.indexing.db.split_once("://").unwrap();
        let p = PathBuf::from(path);
        log::info!("Verifying sqlite path exists at {}", p.display());
        if !tokio::fs::try_exists(p).await? {
            let err = anyhow::anyhow!("Cannot connect sqlite at path {}, aborting startup", path);
            log::error!("{}", err.to_string());
            return Err(err);
        }
    }
    Ok(())
}
