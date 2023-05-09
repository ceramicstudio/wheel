use spinners::{Spinner, Spinners};
use std::path::Path;
use tokio::process::Command;

pub async fn npm_install_package(
    working_directory: impl AsRef<Path>,
    package: &str,
) -> anyhow::Result<()> {
    let version = Command::new("npm").args(&["-v"]).output().await?;
    let ver = semver::Version::parse(String::from_utf8_lossy(&version.stdout).trim())?;
    if ver.major < 9 || (ver.major == 9 && ver.minor <= 5) {
        let status = Command::new("npm")
            .args(&["init", "--yes"])
            .current_dir(working_directory.as_ref())
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("Failed to init npm, cannot download {}", package);
        }
    }

    npm_install(working_directory, &Some(&package)).await?;

    Ok(())
}

pub async fn npm_install(
    working_directory: impl AsRef<Path>,
    package: &Option<&str>,
) -> anyhow::Result<()> {
    let msg = "Installing dependencies";
    let mut args = vec!["install"];
    let msg = if let Some(p) = package {
        args.push(p);
        format!("{} for {}", msg, p)
    } else {
        msg.to_string()
    };
    let mut s = Spinner::new(Spinners::Star2, msg);
    let out = Command::new("npm")
        .args(&args)
        .current_dir(&working_directory)
        .output()
        .await?;
    if !out.status.success() {
        anyhow::bail!("Failed to install app template dependencies");
    }
    s.stop_with_newline();
    Ok(())
}
