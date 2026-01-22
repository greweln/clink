use crate::{GIT_AHEAD, GIT_BEHIND, Result, modules::*, style::Color};
use git2::Repository;

pub trait Render {
    fn render(&self) -> Result<String>;
}

impl Render for Username {
    fn render(&self) -> Result<String> {
        let text = Username::text()?;
        Ok(self.style.apply(&text))
    }
}

impl Render for Hostname {
    fn render(&self) -> Result<String> {
        let text = Hostname::text()?;
        Ok(self.style.apply(&text))
    }
}

impl Render for Pwd {
    fn render(&self) -> Result<String> {
        let text = Pwd::text()?;
        Ok(self.style.apply(&text))
    }
}

impl Render for Permission {
    fn render(&self) -> Result<String> {
        let text = self.text()?;
        if text.is_empty() {
            return Ok(String::new());
        }

        let style = self.style.as_ref().unwrap_or(&Color::Red);
        Ok(style.apply(&text))
    }
}

impl Render for Symbol {
    fn render(&self) -> Result<String> {
        let style = self.style.as_ref().unwrap_or(&Color::Foreground);
        Ok(style.apply(&self.text))
    }
}

impl Render for Custom {
    fn render(&self) -> Result<String> {
        let style = self.style.as_ref().unwrap_or(&Color::Foreground);
        Ok(style.apply(&self.text))
    }
}

impl Render for Fill {
    fn render(&self) -> Result<String> {
        let style = self.style.as_ref().unwrap_or(&Color::Foreground);
        Ok(style.apply(&self.text))
    }
}

impl Render for Git {
    fn render(&self) -> Result<String> {
        let path = std::env::current_dir()?;

        // Early exit if not a git repo
        let repo = match Repository::discover(&path) {
            Ok(r) => r,
            Err(_) => return Ok(String::new()),
        };

        // Handle Unborn/Empty Repo State
        if repo.is_empty().unwrap_or(false) {
            let default_branch = repo
                .config()
                .ok()
                .and_then(|config| config.get_string("init.defaultBranch").ok())
                .unwrap_or_else(|| "master".to_string());

            let style = self.branch_style.as_ref().unwrap_or(&Color::White);
            // Replaced .colorize() with .apply()
            return Ok(style.apply(&format!("{} (unborn)", default_branch)));
        }

        let repo_root = repo
            .path()
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| path.display().to_string());

        let mut prompt = String::new();

        // 1. Branch Name
        if self.branch {
            let branch_text = Self::branch(&repo)?;
            let style = self.branch_style.as_ref().unwrap_or(&Color::White);
            prompt.push_str(&style.apply(&branch_text));
        }

        // 2. Git State (Clean/Dirty)
        if self.state {
            match self.state(&repo)? {
                State::Clean(text) => {
                    let style = self.state_clean_style.as_ref().unwrap_or(&Color::Green);
                    prompt.push_str(&style.apply(&text));
                }
                State::Dirty(text) => {
                    let style = self.state_dirty_style.as_ref().unwrap_or(&Color::Red);
                    prompt.push_str(&style.apply(&text));
                }
            }
        }

        // 3. Ahead/Behind Logic
        if self.ahead_behind {
            let (ahead, behind) = Self::ahead_behind(&repo)?;
            let style = self.ahead_behind_style.as_ref().unwrap_or(&Color::White);

            if ahead > 0 {
                let symbol = self.ahead_symbol.as_deref().unwrap_or(GIT_AHEAD);
                prompt.push_str(&style.apply(&format!(" {}{}", symbol, ahead)));
            }
            if behind > 0 {
                let symbol = self.behind_symbol.as_deref().unwrap_or(GIT_BEHIND);
                prompt.push_str(&style.apply(&format!(" {}{}", symbol, behind)));
            }
        }

        // Save the top-level repo path to cache for the background updater
        save_to_cache(&repo_root)?;

        Ok(prompt)
    }
}
