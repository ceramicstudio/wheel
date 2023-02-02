mod ceramic;
mod prompt;

pub async fn interactive() -> anyhow::Result<()> {
    let project = prompt::project::configure_project().await?;
    let doc = prompt::did::generate_did(&project.path).await?;
    let cfg = prompt::ceramic::prompt(Some(&doc)).await?;
    ceramic::install_ceramic(&project.name, &project.path).await?;

    Ok(())
}
