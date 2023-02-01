mod ceramic;
mod prompt;

pub async fn interactive() -> anyhow::Result<()> {
    let doc = prompt::did::generate_did().await?;
    let cfg = prompt::ceramic::prompt(None).await?;
    ceramic::install_ceramic().await?;
}