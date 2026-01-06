use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    // #[error("Invalid path: {0}")]
    // InvalidPath(String),
    #[error("io: {0}")]
    IoError(#[from] std::io::Error),
    #[error("User configuration error:\n {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Variable not found: {0}")]
    Var(#[from] std::env::VarError),
    //#[error("Invalid color: '{0}'. Expected one of: foreground, black, grey, red, blue, green, yellow, cyan, magenta, white, bright_white"
    //)]
    // InvalidColor(String),
    #[error("Cannot get the home dir")]
    FailedGetHomeDir,
    #[error("{0}")]
    Git(#[from] git2::Error),
    // #[error("{0}")]
    // Serde(#[from] serde_json::Error),
    #[error("{0}")]
    SysTime(#[from] SystemTimeError),
    #[error("Module '{0}' duplicated")]
    DuplicateModule(String),
    #[error("Module '{0}' not defined")]
    ModuleNotDefined(String),
    #[error("Layout is not defined")]
    EmptyLayout,
}
