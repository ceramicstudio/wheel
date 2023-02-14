pub mod ceramic_app_template;
pub mod ceramic_daemon;
pub mod compose_db;
//pub mod kubo;

use std::io::BufRead;

pub fn log_errors(stdout: Vec<u8>) {
    let out = std::io::Cursor::new(stdout);
    for l in std::io::BufReader::new(out).lines() {
        if let Ok(l) = l {
            log::error!("{}", l);
        }
    }
}
