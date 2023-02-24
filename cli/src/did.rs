use ssi::did::{Document, DocumentBuilder, DIDMethod, Source};

pub async fn generate_document() -> anyhow::Result<Document> {
    // let mut vc = ssi::vc::Credential;
    let key = ssi::jwk::JWK::generate_ed25519()?;
    let did = did_method_key::DIDKey.generate(&Source::Key(&key)).ok_or_else(|| anyhow::anyhow!("Failed to generate DID"))?;
    // let verification_method = ssi::vc::get_verification_method(&did, &did_method_key::DIDKey).await?;
    // let mut issue_options = ssi::vc::LinkedDataProofOptions::default();
    // let mut context_loader = ssi::jsonld::ContextLoader::default();
    // issue_options.verification_method = Some(ssi::vc::URI::String(verification_method));
    
    let mut builder = DocumentBuilder::default();
    builder.id(did);
    // builder.public_key(key);
    let doc = builder.build().map_err(|e| anyhow::anyhow!(e))?;
    log::info!("Generated DID: {}", doc.id);
    Ok(doc)
}