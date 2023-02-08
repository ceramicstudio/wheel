use futures_util::StreamExt;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

async fn fetch_url(url: String, file: &Path) -> anyhow::Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = tokio::fs::File::create(file).await?;
    let mut bytes = response.bytes_stream();
    while let Some(b) = bytes.next().await {
        let b = b?;
        file.write_all(&b).await?;
    }
    file.flush().await?;
    Ok(())
}

//https://dist.ipfs.tech/kubo/v0.18.1/kubo_v0.18.1_darwin-amd64.tar.gz
#[cfg(all(target_arch = "x86_64", target_os = "macos"))]
const ARCH: &'static str = "darwin-amd64";
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
const ARCH: &'static str = "linux-amd64";
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
const ARCH: &'static str = "darwin-arm64";

pub async fn install_kubo(
    working_directory: &Path,
    version: &Option<semver::Version>,
) -> anyhow::Result<()> {
    log::info!("Checking for kubo");
    if !Command::new("command")
        .args(&["-v", "kubo"])
        .status()
        .await?
        .success()
    {
        let version = version
            .clone()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "0.18.1".to_string());
        let url = format!(
            "https://dist.ipfs.tech/kubo/v{}/kubo_v{}_{}.tar.gz",
            version, version, ARCH
        );
        let path = working_directory.join("kubo.tar.gz");
        fetch_url(url, &path).await?;
        if !Command::new("tar")
            .args(&["-xvf", &path.to_string_lossy()])
            .status()
            .await?
            .success()
        {
            anyhow::bail!("Failed to extract kubo")
        }
    }

    Ok(())
}
