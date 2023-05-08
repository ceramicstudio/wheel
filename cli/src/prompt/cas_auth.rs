use crate::did::DidAndPrivateKey;
use ceramic_config::{CasAuth, NetworkIdentifier};
use inquire::{Select, Text};
use serde::Deserialize;

#[derive(Deserialize)]
struct CasValidResponse {
    email: String,
    did: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CasResponse {
    Invalid { error: String },
    Valid(Vec<CasValidResponse>),
}

enum CasSelect {
    Authenticate,
    Ip,
}

impl std::fmt::Display for CasSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authenticate => write!(f, "Email Based Authentication"),
            Self::Ip => write!(f, "IP Based Authentication (Deprecated)"),
        }
    }
}

pub async fn prompt(
    doc: &DidAndPrivateKey,
    id: &NetworkIdentifier,
) -> anyhow::Result<Option<CasAuth>> {
    let url = match id {
        NetworkIdentifier::InMemory => None,
        NetworkIdentifier::Local | NetworkIdentifier::Dev => {
            Some("https://cas-qa.3boxlabs.com".to_string())
        }
        NetworkIdentifier::Clay => Some("https://cas-clay.3boxlabs.com".to_string()),
        NetworkIdentifier::Mainnet => Some("https://cas.3boxlabs.com".to_string()),
    };
    if let Some(url) = url {
        let url = Text::new("CAS Url").with_default(&url).prompt()?;
        let pk = match Select::new(
            "CAS Authentication",
            vec![
                CasSelect::Authenticate,
                CasSelect::Ip,
            ],
        ).with_help_message("For more information on CAS Authentication see https://composedb.js.org/docs/0.4.x/guides/composedb-server/access-mainnet#step-1-start-your-node-and-copy-your-key-did")
        .prompt()?
        {
            CasSelect::Authenticate => {
                let input_email = Text::new("Email address for CAS Authentication").prompt()?;
                log::info!("Sending OTP to {}, please check your email", input_email);
                reqwest::Client::new()
                    .post(format!("{}/api/v0/auth/verification", url))
                    .json(&serde_json::json!({
                        "email": input_email,
                    }))
                    .send()
                    .await?;
                let code = Text::new("OTP Code from email")
                    .with_help_message("Please check your email for the OTP code")
                    .prompt()?;
                let input_did = doc.did();
                let bytes = reqwest::Client::new()
                    .post(format!("{}/api/v0/auth/did", url))
                    .json(&serde_json::json!({
                        "email": input_email,
                        "otp": code,
                        "dids": [input_did],
                    }))
                    .send()
                    .await?
                    .bytes()
                    .await?;
                match serde_json::from_slice::<CasResponse>(&bytes) {
                    Err(_) => {
                        anyhow::bail!(
                            "CAS authentication failed: {}",
                            String::from_utf8_lossy(&bytes)
                        );
                    }
                    Ok(CasResponse::Invalid { error }) => {
                        anyhow::bail!("CAS authentication failed: {}", error);
                    }
                    Ok(CasResponse::Valid(responses)) => {
                        if let Some(resp) = responses.first() {
                            if resp.email != input_email && resp.did != input_did {
                                anyhow::bail!("CAS response did not match email and did");
                            }
                            log::info!("CAS authentication successful");
                            Some(doc.cas_auth())
                        } else {
                            anyhow::bail!(
                                "CAS authentication failed: {}",
                                String::from_utf8_lossy(&bytes)
                            );
                        }
                    }
                }
            }
            CasSelect::Ip => None,
        };
        Ok(Some(CasAuth { url: url, pk: pk }))
    } else {
        Ok(None)
    }
}
