pub mod ceramic_app_template;
pub mod ceramic_daemon;
pub mod compose_db;
mod npm;
mod verify_db;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

pub async fn log_async_errors<T>(out: T)
where
    T: tokio::io::AsyncRead + Unpin,
{
    let mut lines = tokio::io::BufReader::new(out).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        log::error!("    {}", line.trim())
    }
}

pub async fn create_invoke_script(
    path_to_cmd: impl AsRef<Path>,
    path_to_script: impl AsRef<Path>,
    pre: &str,
) -> anyhow::Result<()> {
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .append(false)
        .open(path_to_script.as_ref())
        .await
        .unwrap();
    f.write_all(
        format!(
            r#"
#!/usr/bin/env sh
{}
node {} "$@"
"#,
            pre,
            path_to_cmd.as_ref().display(),
        )
        .as_bytes(),
    )
    .await?;
    f.flush().await?;
    tokio::fs::set_permissions(path_to_script, std::fs::Permissions::from_mode(0o755)).await?;
    Ok(())
}
