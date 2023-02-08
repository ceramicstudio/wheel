pub mod ceramic_advanced_config;
pub mod ceramic_local_config;
pub mod ceramic_remote_config;
pub mod did;
pub mod project;

pub use ceramic_advanced_config::{configure as advanced_config, prompt};
pub use ceramic_local_config::configure as local_config;
pub use ceramic_remote_config::configure as remote_config;
