use inquire::*;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use crate::did::DidAndPrivateKey;

enum DidSelect {
    Generate,
    Input,
}

impl std::fmt::Display for DidSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generate => write!(f, "Generate"),
            Self::Input => write!(f, "Input"),
        }
    }
}

pub async fn generate_did(path: impl AsRef<Path>) -> anyhow::Result<DidAndPrivateKey> {
    let ans = Select::new(
        "Admin DID Configuration",
        vec![DidSelect::Generate, DidSelect::Input],
    )
    .with_help_message("Whether DID should be input from file, or generated")
    .prompt()?;

    let default_admin_key_location = path.as_ref().join("admin.json");

    let doc = match ans {
        DidSelect::Generate => {
            let doc = DidAndPrivateKey::generate()?;
            if let Some(p) = Text::new("File to save DID private key to? (Escape to skip)")
                .with_default(&path.as_ref().join("admin.pk").to_string_lossy())
                .prompt_skippable()?
            {
                let mut opts = tokio::fs::OpenOptions::new();
                opts.write(true).create(true).append(false);
                let mut f = opts.open(p).await?;
                f.write(doc.pk().as_bytes()).await?;
                f.flush().await?;
            }
            Ok(doc)
        }
        DidSelect::Input => {
            let k = Password::new("Admin DID Private Key").prompt()?;
            let p = Text::new("Path to Admin DID File")
                .with_default(default_admin_key_location.to_string_lossy().as_ref())
                .prompt()?;
            let data = tokio::fs::read(PathBuf::from(p)).await?;
            let doc: ssi::did::Document = serde_json::from_slice(&data)?;
            Ok(DidAndPrivateKey::new(k, doc))
        }
    };

    doc
}
