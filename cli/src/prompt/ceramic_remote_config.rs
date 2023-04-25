use ceramic_config::*;
use std::path::Path;

use crate::did::DidAndPrivateKey;

pub async fn configure<'a, 'b, P: AsRef<Path>>(
    working_directory: P,
    cfg: &'a mut Config,
    admin_did: &'b DidAndPrivateKey,
) -> anyhow::Result<()> {
    super::ceramic_advanced_config::configure_ipfs(cfg)?;
    super::ceramic_advanced_config::configure_state_store(cfg).await?;
    super::ceramic_advanced_config::configure_http_api(cfg, admin_did)?;
    super::ceramic_advanced_config::configure_node(cfg)?;
    super::ceramic_advanced_config::configure_indexing(working_directory, cfg)?;
    super::ceramic_advanced_config::configure_anchor(cfg)?;
    Ok(())
}
