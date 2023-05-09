use std::path::Path;
use tokio::process::Command;

pub async fn npm_install(working_directory: impl AsRef<Path>, package: &str) -> anyhow::Result<()> {
    let version = Command::new("npm").args(&["-v"]).output().await?;
    let ver = semver::Version::parse(String::from_utf8_lossy(&version.stdout).trim())?;
    if ver.major < 9 {
        let status = Command::new("npm")
            .args(&["init", "--yes"])
            .current_dir(working_directory.as_ref())
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("Failed to init npm, cannot download {}", package);
        }
    }

    let status = Command::new("npm")
        .args(&["install", package])
        .current_dir(working_directory)
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("Failed to install {}", package);
    }

    Ok(())
}
