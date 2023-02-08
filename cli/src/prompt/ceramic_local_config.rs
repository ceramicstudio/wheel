use ceramic_config::*;
use inquire::*;
use ssi::did::Document;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub async fn prompt(working_directory: &Path, admin_did: &Document) -> anyhow::Result<Config> {
    let cfg_file_path = working_directory.join("ceramic.json");
    let cfg_file_path = Text::new("Ceramic ceramic-config file location")
        .with_default(cfg_file_path.to_string_lossy().as_ref())
        .prompt()?;
    let cfg_file_path = PathBuf::from(cfg_file_path);
    let (mut cfg, existing) = if cfg_file_path.exists() {
        let data = tokio::fs::read(cfg_file_path.clone()).await?;
        let cfg = serde_json::from_slice(data.as_slice())?;
        (cfg, " Existing configuration will be overwritten")
    } else {
        (Config::default(), "")
    };

    let ans = Confirm::new(&format!("Start ceramic configuration?{}", existing))
        .with_help_message("Step through interactive prompts to configure ceramic node")
        .prompt_skippable()?;

    if let Some(true) = ans {
        configure_ceramic(&mut cfg, admin_did).await?;
    }

    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(cfg_file_path)
        .await?;
    f.write_all(serde_json::to_string(&cfg)?.as_bytes()).await?;
    f.flush().await?;

    Ok(cfg)
}

fn configure_http_api(admin_did: &Document) -> anyhow::Result<HttpApi> {
    let mut http = HttpApi::default();
    http.hostname = Text::new("Bind address")
        .with_default(&http.hostname)
        .prompt()?;
    http.port = Text::new("Bind port")
        .with_default(&http.port.to_string())
        .prompt()?
        .parse()?;
    let cors = Text::new("Cors origins, comma separated").prompt()?;
    let cors = cors.split(",").map(|s| s.trim().to_string()).collect();
    http.cors_allowed_origins = cors;
    http.admin_dids = vec![admin_did.id.clone()];
    Ok(http)
}

fn configure_indexing() -> anyhow::Result<Indexing> {
    let mut index = Indexing::default();
    index.db = Text::new("Database Url").with_default(&index.db).prompt()?;
    Ok(index)
}

async fn configure_ceramic<'a, 'b>(
    cfg: &'a mut Config,
    admin_did: &'b Document,
) -> anyhow::Result<&'a mut Config> {
    cfg.http_api = configure_http_api(admin_did)?;
    cfg.network = Network::clay();
    cfg.anchor = Anchor::clay();
    cfg.indexing = configure_indexing()?;

    Ok(cfg)
}
