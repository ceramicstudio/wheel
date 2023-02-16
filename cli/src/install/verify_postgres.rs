use ceramic_config::Config;
use sqlx::Connection;

pub async fn verify_postgres(cfg: &Config) -> anyhow::Result<()> {
    if cfg.indexing.db.starts_with("postgresql") {
        sqlx::postgres::PgConnection::connect(&cfg.indexing.db).await?;
    }
    Ok(())
}
