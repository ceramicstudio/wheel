use ceramic_config::*;
use inquire::*;
use ssi::did::Document;

fn configure_anchor(cfg: &mut Config) -> anyhow::Result<()> {
    cfg.anchor.ethereum_rpc_url = Text::new("Ethereum RPC Url")
        .with_default(&cfg.anchor.ethereum_rpc_url)
        .prompt()?;
    Ok(())
}

pub async fn configure<'a, 'b>(cfg: &'a mut Config, admin_did: &'b Document) -> anyhow::Result<()> {
    super::ceramic_advanced_config::configure_ipfs(cfg)?;
    super::ceramic_advanced_config::configure_state_store(cfg).await?;
    super::ceramic_advanced_config::configure_http_api(cfg, admin_did)?;
    super::ceramic_advanced_config::configure_node(cfg)?;
    super::ceramic_advanced_config::configure_indexing(cfg)?;
    configure_anchor(cfg)
}
