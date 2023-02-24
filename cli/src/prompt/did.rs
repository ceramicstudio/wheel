use inquire::*;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

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

pub async fn generate_did(path: &Path) -> anyhow::Result<ssi::did::Document> {
    let ans = Select::new(
        "Admin DID Configuration",
        vec![DidSelect::Generate, DidSelect::Input],
    )
    .with_help_message("Step through interactive prompts to configure ceramic node")
    .prompt()?;

    let default_admin_key_location = path.join("admin.json");

    let doc = match ans {
        DidSelect::Generate => {
            let doc = crate::did::generate_document().await?;
            if let Some(p) = Text::new("Location to save DID to? (Escape to skip)")
                .with_default(default_admin_key_location.to_string_lossy().as_ref())
                .prompt_skippable()?
            {
                let p = PathBuf::from(p);
                let mut f = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(p)
                    .await?;
                f.write_all(serde_json::to_string(&doc)?.as_bytes()).await?;
                f.flush().await?;
            }
            doc
        }
        DidSelect::Input => {
            let p = Text::new("Path to Admin DID File")
                .with_default(default_admin_key_location.to_string_lossy().as_ref())
                .prompt()?;
            let data = tokio::fs::read(PathBuf::from(p)).await?;
            let doc: ssi::did::Document = serde_json::from_slice(&data)?;
            doc
        }
    };

    Ok(doc)
}
