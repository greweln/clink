use crate::{CURRENT_UID, DENIED, Result, SYMBOL, errors::Errors, style::Color};
use crate::{GIT_AHEAD, GIT_BEHIND, GIT_CLEAN, GIT_DIRTY};
use ansi_str::AnsiStr;
use dirs::home_dir;
use fs2::FileExt;
use git2::{ErrorCode, Repository};
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::{
    env,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use terminal_size::{Width, terminal_size};

// ********************
// ===== USERNAME ====
// ********************
#[derive(Deserialize, Debug, Clone)]
pub struct Username {
    pub style: Color,
}
impl Username {
    pub fn text() -> Result<String> {
        let username = env::var("USER")?;
        Ok(username)
    }
}

impl Default for Username {
    fn default() -> Self {
        Self {
            style: Color::Foreground,
        }
    }
}

// ********************
// ===== HOSTNAME ====
// ********************
#[derive(Deserialize, Debug, Clone)]
pub struct Hostname {
    pub style: Color,
}
impl Hostname {
    pub fn text() -> Result<String> {
        let hostname = env::var("HOSTNAME")?;
        Ok(hostname)
    }
}

impl Default for Hostname {
    fn default() -> Self {
        Self {
            style: Color::Foreground,
        }
    }
}

// ********************
//   ===== PWD ====
// ********************
#[derive(Deserialize, Debug)]
pub struct Pwd {
    pub style: Color,
}

impl Pwd {
    pub fn text() -> Result<String> {
        let icon = "~";

        // 1. Get current path safely
        let current_dir = match env::var("PWD") {
            Ok(v) => PathBuf::from(v),
            Err(_) => env::current_dir()?,
        };

        // 2. Get home dir safely
        let home_dir = home_dir().ok_or(Errors::FailedGetHomeDir)?;

        // 3. Perform Tilde Substitution
        let pwd = if current_dir.starts_with(&home_dir) {
            let relative = current_dir
                .strip_prefix(&home_dir)
                .map_err(|_| Errors::FailedGetHomeDir)?; // Logical safety

            if relative.as_os_str().is_empty() {
                icon.to_string()
            } else {
                format!("{}/{}", icon, relative.display())
            }
        } else {
            current_dir.display().to_string()
        };

        Ok(pwd)
    }
}

impl Default for Pwd {
    fn default() -> Self {
        Self {
            style: Color::Foreground,
        }
    }
}

// ********************
//  ==== PERMISSION ===
// ********************

#[derive(Deserialize, Debug)]
pub struct Permission {
    text: String,
    pub style: Option<Color>,
}

impl Permission {
    pub fn text(&self) -> Result<String> {
        let path = std::env::current_dir()?;

        // Root user: skip RO indicator
        if *CURRENT_UID == 0 {
            return Ok(String::new());
        }

        let c_path = CString::new(path.as_os_str().as_bytes())?;

        let writable = unsafe { libc::access(c_path.as_ptr(), libc::W_OK) == 0 };

        if writable {
            Ok(String::new())
        } else {
            Ok(self.text.to_string())
        }
    }
}

impl Default for Permission {
    fn default() -> Self {
        Self {
            text: DENIED.into(),
            style: Some(Color::Red),
        }
    }
}

// ********************
// ===== SYMBOL ====
// ********************
#[derive(Deserialize, Debug)]
pub struct Symbol {
    pub text: String,
    pub style: Option<Color>,
}

impl Default for Symbol {
    fn default() -> Self {
        Self {
            text: SYMBOL.into(),
            style: Some(Color::Foreground),
        }
    }
}

// ********************
// ===== CUSTOM ====
// ********************
#[derive(Deserialize, Debug)]
pub struct Custom {
    pub text: String,
    pub style: Option<Color>,
}

// ********************
// ===== FILL ====
// ********************
#[derive(Deserialize, Debug)]
pub struct Fill {
    pub text: String,
    pub style: Option<Color>,
}

impl Fill {
    /// Expands the fill character to occupy the remaining horizontal space
    /// on the current terminal line, accounting for hidden ANSI and shell markers.
    pub fn expand(&self, current_prompt: &str) -> Self {
        // Get the length of the prompt
        let prompt_len = current_prompt
            .ansi_strip()
            .chars()
            .filter(|&c| c != '\x01' && c != '\x02')
            .count();

        // Detect terminal width, fallback to 80
        let terminal_width = terminal_size()
            .map(|(Width(w), _)| w as usize)
            .unwrap_or(80);

        // How many fill chars to add
        let dash_count = terminal_width.saturating_sub(prompt_len);

        // Repeat dash symbol
        let fill = self.text.clone().repeat(dash_count);

        Self {
            text: fill,
            style: self.style.clone(),
        }
    }
}

impl Default for Fill {
    fn default() -> Self {
        Self {
            text: "-".into(),
            style: Some(Color::Foreground),
        }
    }
}

// ********************
// ===== GIT ====
// ********************
#[derive(Serialize, Debug, Clone, Deserialize)]
pub enum State {
    Dirty(String),
    Clean(String),
}

// Write the cache file in a format of repo:timestamp which the update_cache.py can read each
// repo and fetch them.
pub fn save_to_cache(repo_path: &str) -> Result<()> {
    let home = home_dir().unwrap().display().to_string();
    let path = PathBuf::from(format!("{}/.cache/clink", home));

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let entry = format!("{}:{}", repo_path, timestamp);

    let mut file = OpenOptions::new().append(true).create(true).open(&path)?;

    // Try to acquire an exclusive flock() lock before writing to the cache file.
    // This prevents concurrent prompt instances or background updaters from writing
    // at the same time. If the file is already locked, we skip the write to avoid
    // blocking the prompt. (Uses fs2::FileExt::try_lock_exclusive and unlock.)
    if file.try_lock_exclusive().is_err() {
        return Ok(());
    }

    writeln!(file, "{}", entry)?;
    file.flush()?;
    fs2::FileExt::unlock(&file)?;

    Ok(())
}

#[derive(Debug, Deserialize, Clone)]
pub struct Git {
    pub branch: bool,
    pub branch_style: Option<Color>,
    pub state: bool,
    pub state_clean: Option<String>,
    pub state_clean_style: Option<Color>,
    pub state_dirty: Option<String>,
    pub state_dirty_style: Option<Color>,
    pub state_style: Option<Color>,
    pub ahead_behind: bool,
    pub ahead_behind_style: Option<Color>,
    pub ahead_symbol: Option<String>,
    pub behind_symbol: Option<String>,
}

impl Default for Git {
    fn default() -> Self {
        Self {
            branch: true,
            branch_style: Some(Color::Foreground),
            state: true,
            state_clean: Some(GIT_CLEAN.into()),
            state_clean_style: Some(Color::Green),
            state_dirty: Some(GIT_DIRTY.into()),
            state_dirty_style: Some(Color::Red),
            state_style: Some(Color::Foreground),
            ahead_behind: true,
            ahead_behind_style: Some(Color::Foreground),
            ahead_symbol: Some(GIT_AHEAD.into()),
            behind_symbol: Some(GIT_BEHIND.into()),
        }
    }
}

impl Git {
    pub fn branch(repo: &Repository) -> Result<String> {
        // Try to get the HEAD reference.
        let head = match repo.head() {
            // SUCCESS: We have a HEAD reference. Continue as normal.
            Ok(h) => h,

            // FAILURE: Check if the failure is the UnbornBranch error (ErrorCode::UnbornBranch).
            // Although render() should prevent this, it's a good safeguard.
            Err(e) if e.code() == ErrorCode::UnbornBranch => {
                // If it's an unborn branch, return a safe, descriptive string.
                let default_branch = repo
                    .config()
                    .ok()
                    .and_then(|config| config.get_string("init.defaultBranch").ok())
                    .unwrap_or_else(|| "master".to_string());

                return Ok(format!("{} (unborn)", default_branch));
            }

            // Other Errors: Propagate them up to the caller (e.g., corruption, permissions).
            Err(e) => return Err(e.into()),
        };

        // If successful, determine if it's a branch or a detached HEAD.
        let branch = if head.is_branch() {
            // It's a named branch, get the shorthand (e.g., "master")
            head.shorthand()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            // Detached HEAD — show short commit hash
            head.target()
                .map(|oid| format!("@{}", &oid.to_string()[..7]))
                .unwrap_or_else(|| "detached".to_string())
        };

        Ok(branch)
    }

    ///
    /// Returns the repo state as a string (empty if clean, or specified string if dirty)
    ///
    pub fn state(&self, repo: &Repository) -> Result<State> {
        // Get repository statuses
        let statuses = match repo.statuses(None) {
            Ok(s) => s,
            Err(_) => return Ok(State::Clean(GIT_CLEAN.to_string())), // If we can't read statuses, treat as clean
        };

        // If any file is changed (staged or unstaged), consider repo dirty
        // It's clean once it's commited
        if statuses.iter().any(|entry| {
            let s = entry.status();
            s.intersects(
                git2::Status::WT_MODIFIED
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::WT_NEW
                    | git2::Status::INDEX_NEW
                    | git2::Status::WT_DELETED
                    | git2::Status::INDEX_DELETED,
            )
        }) {
            Ok(State::Dirty(
                self.state_dirty.clone().unwrap_or(GIT_DIRTY.into()),
            ))
        } else {
            Ok(State::Clean(
                self.state_clean.clone().unwrap_or(GIT_CLEAN.into()),
            )) // clean repo
        }
    }

    // Return how many ahead/behind commits exist.
    pub fn ahead_behind(repo: &Repository) -> Result<(u32, u32)> {
        let head = repo.head()?; // current branch reference
        let branch = head.resolve()?; // follow symbolic refs

        // Only proceed if branch has an upstream
        let upstream_oid = match repo.branch_upstream_name(branch.name().unwrap()) {
            Ok(upstream_name_buf) => {
                let upstream_str = upstream_name_buf.as_str().unwrap_or("?");
                let upstream_ref = repo.revparse_single(upstream_str)?;
                upstream_ref.id()
            }
            Err(_) => return Ok((0, 0)), // no upstream set
        };

        let (ahead, behind) = repo.graph_ahead_behind(branch.target().unwrap(), upstream_oid)?;

        Ok((ahead as u32, behind as u32))
    }
}
