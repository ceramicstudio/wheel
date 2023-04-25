use ssi::{
    did::{DIDMethod, Document, DocumentBuilder, Source},
    jwk::Params,
};

pub struct DidAndPrivateKey {
    private_key: String,
    document: Document,
}

impl DidAndPrivateKey {
    pub fn new(private_key: String, doc: Document) -> Self {
        Self {
            private_key: private_key,
            document: doc,
        }
    }

    pub fn cas_auth(&self) -> String {
        format!("inplace:ed25519#{}", self.private_key)
    }

    pub fn did(&self) -> &str {
        &self.document.id
    }

    pub fn pk(&self) -> &str {
        &self.private_key
    }

    pub fn generate() -> anyhow::Result<DidAndPrivateKey> {
        // let mut vc = ssi::vc::Credential;
        let key = ssi::jwk::JWK::generate_ed25519()?;
        let private_key = if let Params::OKP(params) = &key.params {
            let pk = params
                .private_key
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("No private key"))?;
            hex::encode(pk.0.as_slice())
        } else {
            anyhow::bail!("Invalid private key");
        };
        let did = did_method_key::DIDKey
            .generate(&Source::Key(&key))
            .ok_or_else(|| anyhow::anyhow!("Failed to generate DID"))?;
        // let verification_method = ssi::vc::get_verification_method(&did, &did_method_key::DIDKey).await?;
        // let mut issue_options = ssi::vc::LinkedDataProofOptions::default();
        // let mut context_loader = ssi::jsonld::ContextLoader::default();
        // issue_options.verification_method = Some(ssi::vc::URI::String(verification_method));

        let mut builder = DocumentBuilder::default();
        builder.id(did);
        // builder.public_key(key);
        let doc = builder.build().map_err(|e| anyhow::anyhow!(e))?;
        log::info!("Generated DID: {}", doc.id);
        Ok(DidAndPrivateKey {
            private_key: private_key,
            document: doc,
        })
    }
}
