use spinners::{Spinner, Spinners};
use std::io::BufRead;
use std::path::Path;
use tokio::process::Command;

pub async fn npm_install_package(
    working_directory: impl AsRef<Path>,
    package: &str,
    globally: bool,
) -> anyhow::Result<()> {
    let status = Command::new("npm")
        .args(&["init", "--yes"])
        .current_dir(working_directory.as_ref())
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("Failed to init npm, cannot download {}", package);
    }

    npm_install(working_directory, &Some(&package), globally).await?;

    Ok(())
}

pub async fn npm_install(
    working_directory: impl AsRef<Path>,
    package: &Option<&str>,
    globally: bool,
) -> anyhow::Result<()> {
    let msg = "Installing dependencies";
    let mut args = vec!["install"];
    if globally {
        args.push("-g");
    }
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
        log::error!("Failed to install");
        let b = std::io::BufReader::new(std::io::Cursor::new(out.stdout));
        for l in b.lines() {
            if let Ok(l) = l {
                log::error!("{l}");
            }
        }
        let b = std::io::BufReader::new(std::io::Cursor::new(out.stderr));
        for l in b.lines() {
            if let Ok(l) = l {
                log::error!("err: {l}");
            }
        }
        anyhow::bail!("Failed to install, please see output");
    }
    s.stop_with_newline();
    Ok(())
}
