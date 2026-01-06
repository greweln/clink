use once_cell::sync::Lazy;
use users::{get_current_gid, get_current_uid};

pub type Result<T> = std::result::Result<T, errors::Errors>;
pub mod config;
pub mod errors;
pub mod modules;
pub mod render;
pub mod style;

pub const DENIED: &str = "X";
pub const SYMBOL: &str = ">";
pub const FILL: &str = "-";

// These are computed only once, on first use
pub static CURRENT_UID: Lazy<u32> = Lazy::new(get_current_uid);
pub static CURRENT_GID: Lazy<u32> = Lazy::new(get_current_gid);

pub const IGNORE_START: &str = "\x01";
pub const IGNORE_END: &str = "\x02";
pub const ANSI_RESET: &str = "\x1b[0m";

pub const GIT_CLEAN: &str = "";
pub const GIT_DIRTY: &str = "!";
pub const GIT_AHEAD: &str = "↑";
pub const GIT_BEHIND: &str = "↓";
