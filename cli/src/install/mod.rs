pub mod ceramic_app_template;
pub mod ceramic_daemon;
pub mod compose_db;
//pub mod kubo;
mod verify_postgres;

use std::io::BufRead;
use tokio::io::AsyncBufReadExt;

pub fn log_errors<T>(out: T)
where
    T: AsRef<[u8]>,
{
    let out = std::io::Cursor::new(out);
    for l in std::io::BufReader::new(out).lines() {
        if let Ok(l) = l {
            log::error!("    {}", l.trim());
        }
    }
}

pub async fn log_async_errors<T>(out: T)
where
    T: tokio::io::AsyncRead + Unpin,
{
    let mut lines = tokio::io::BufReader::new(out).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        log::error!("    {}", line.trim())
    }
}
