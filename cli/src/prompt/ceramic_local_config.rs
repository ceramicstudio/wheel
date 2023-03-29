use ceramic_config::*;

use crate::did::DidAndPrivateKey;

pub async fn configure<'a, 'b>(
    cfg: &'a mut Config,
    admin_did: &'b DidAndPrivateKey,
) -> anyhow::Result<()> {
    super::ceramic_advanced_config::configure_http_api(cfg, admin_did)?;
    super::ceramic_advanced_config::configure_indexing(cfg)?;
    super::ceramic_advanced_config::configure_anchor(cfg)?;
    Ok(())
}
