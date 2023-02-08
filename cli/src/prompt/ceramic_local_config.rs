use ceramic_config::*;
use ssi::did::Document;

pub async fn configure<'a, 'b>(cfg: &'a mut Config, admin_did: &'b Document) -> anyhow::Result<()> {
    super::ceramic_advanced_config::configure_http_api(cfg, admin_did)?;
    super::ceramic_advanced_config::configure_indexing(cfg)?;

    Ok(())
}
