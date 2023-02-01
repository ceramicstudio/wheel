use ed25519_compact::{KeyPair, Seed};
use hex::ToHex;
use inquire::*;
use ssi::did::DocumentBuilder;
use std::path::PathBuf;
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

const DID_DEFAULT_LOCATION: &'static str = "/etc/ceramic/admin.did";

pub async fn generate_did() -> anyhow::Result<ssi::did::Document> {
    let ans = Select::new(
        "Admin DID Configuration",
        vec![DidSelect::Generate, DidSelect::Input, DidSelect::Exit],
    )
    .with_help_message("Step through interactive prompts to configure ceramic node")
    .prompt()?;

    let doc = match ans {
        DidSelect::Generate => {
            let p = Text::new("Location to save keypair to")
                .with_default("/etc/ceramic/generated")
                .prompt()?;
            let p = PathBuf::from(p);
            let key_pair = KeyPair::from_seed(Seed::default());
            let mut f = tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(p.clone())
                .await?;
            f.write_all(&*key_pair.sk).await?;
            f.flush().await?;
            let mut f = tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(p.join(".pub"))
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
            let p = Text::new("Path to DID File")
                .with_default("/etc/ceramic/ceramic.json")
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
        .with_default(DID_DEFAULT_LOCATION)
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
