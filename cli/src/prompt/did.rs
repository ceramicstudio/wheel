use inquire::*;
use std::path::{Path, PathBuf};

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

pub async fn generate_did(path: &Path) -> anyhow::Result<DidAndPrivateKey> {
    let ans = Select::new(
        "Admin DID Configuration",
        vec![DidSelect::Generate, DidSelect::Input],
    )
    .with_help_message("Step through interactive prompts to configure ceramic node")
    .prompt()?;

    let default_admin_key_location = path.join("admin.json");

    let doc = match ans {
        DidSelect::Generate => crate::did::DidAndPrivateKey::generate(),
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
