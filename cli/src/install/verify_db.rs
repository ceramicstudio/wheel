use ceramic_config::Config;
use sqlx::Connection;

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
        if let Err(e) = sqlx::sqlite::SqliteConnection::connect(&cfg.indexing.db).await {
            log::error!(
                "Failed to connect to sqlite, aborting daemon startup\n    {}",
                e
            );
            return Err(e.into());
        }
    }
    Ok(())
}
