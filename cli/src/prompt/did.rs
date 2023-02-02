use ed25519_compact::{KeyPair, Seed};
use hex::ToHex;
use inquire::*;
use ssi::did::DocumentBuilder;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

enum DidSelect {
    Generate,
    Input,
    Exit,
}

impl std::fmt::Display for DidSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generate => write!(f, "Generate"),
            Self::Input => write!(f, "Input"),
            Self::Exit => write!(f, "Exit"),
        }
    }
}

pub async fn generate_did(path: &Path) -> anyhow::Result<ssi::did::Document> {
    let ans = Select::new(
        "Admin DID Configuration",
        vec![DidSelect::Generate, DidSelect::Input, DidSelect::Exit],
    )
    .with_help_message("Step through interactive prompts to configure ceramic node")
    .prompt()?;

    let default_admin_key_location = path.join("admin.json");
    let path_str = default_admin_key_location.to_string_lossy();

    let doc = match ans {
        DidSelect::Generate => {
            let p = Text::new("Location to save keypair to")
                .with_default(path.to_string_lossy().as_ref())
                .prompt()?;
            let p = PathBuf::from(p);
            if !p.exists() {
                tokio::fs::create_dir_all(&p).await?
            }
            let key_pair = KeyPair::from_seed(Seed::default());
            let mut f = tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(p.join("ed25519"))
                .await?;
            f.write_all(&*key_pair.sk).await?;
            f.flush().await?;
            let mut f = tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(p.join("ed25519.pub"))
                .await?;
            f.write_all(&*key_pair.pk).await?;
            f.flush().await?;
            DocumentBuilder::default()
                .id(format!(
                    "did:pkh:{}",
                    (&*key_pair.pk).encode_hex::<String>()
                ))
                .build()
        }
        DidSelect::Input => {
            let p = Text::new("Path to Admin DID File")
                .with_default(default_admin_key_location.to_string_lossy().as_ref())
                .prompt()?;
            let data = tokio::fs::read(PathBuf::from(p)).await?;
            DocumentBuilder::default()
                .id(data.encode_hex::<String>())
                .build()
        }
        DidSelect::Exit => {
            log::info!("Exiting");
            std::process::exit(0);
        }
    };

    let doc = doc.map_err(|s| anyhow::anyhow!(s))?;

    if let Some(p) = Text::new("Location to save DID to")
        .with_default(path_str.as_ref())
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

    Ok(doc)
}
