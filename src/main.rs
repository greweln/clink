use clink::config;
use std::path::PathBuf;

fn config_path() -> String {
    // Priority 1: XDG_CONFIG_HOME
    // Priority 2: $HOME/.config
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").expect("HOME environment variable not set");
            PathBuf::from(home).join(".config")
        });

    base.join("clink").join("config.toml").display().to_string()
}

fn main() {
    let path = config_path();

    // If any errors print them and fall back to a usable prompt.
    if let Err(err) = config::run(path) {
        eprintln!("\r\n[clink error]: {err}");

        print!("$ ");
    }
}
