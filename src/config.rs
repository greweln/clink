use crate::{Result, errors::Errors, modules::*, render::Render};

use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fmt, fs,
};

#[derive(Deserialize, Debug, Eq, Hash, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Module {
    Username,
    Pwd,
    Fill,
    Symbol,
    Cursor,
    Permission,
    Git,
    // The "untagged" fallback: if it's not one of the above,
    // catch the string as a "Custom" name.
    #[serde(untagged)]
    Custom(String),
}

// Implement Display so we can convert the Enum to a String easily
impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Module::Custom(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self).map(|_| ()), // Uses the variant name
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    layout: Vec<Module>,
    username: Option<Username>,
    pwd: Option<Pwd>,
    git: Option<Git>,
    fill: Option<Fill>,
    symbol: Option<Symbol>,
    permission: Option<Permission>,
    #[serde(flatten, default)]
    custom: HashMap<String, Custom>,
}

impl Config {
    fn validate_user_config(&self) -> Result<()> {
        // 1. Ensure the layout is not empty
        if self.layout.is_empty() {
            return Err(Errors::EmptyLayout);
        }

        let mut seen: HashSet<&Module> = HashSet::new();

        for module in &self.layout {
            // 2.Check if a custom module in layout is defined.
            if let Module::Custom(name) = module {
                if !self.custom.contains_key(name) {
                    return Err(Errors::ModuleNotDefined(name.to_string()));
                }
            }
            // 3.Ensure Fill, Cursor and Symbol modules are not duplicated

            if matches!(module, Module::Fill | Module::Cursor | Module::Symbol) {
                if !seen.insert(module) {
                    return Err(Errors::DuplicateModule(module.to_string()));
                }
            }
        }

        Ok(())
    }

    fn render(&self) -> Result<String> {
        let mut prompt = String::new();

        // 1. Process the main layout modules (Username, Pwd, etc.)
        for module in &self.layout {
            let output = match module {
                // map_or takes (default_value, closure_for_success)
                Module::Username => self
                    .username
                    .as_ref()
                    .map_or(Username::default().render(), |m| m.render())?,

                Module::Pwd => self
                    .pwd
                    .as_ref()
                    .map_or(Pwd::default().render(), |m| m.render())?,

                Module::Permission => self
                    .permission
                    .as_ref()
                    .map_or(Permission::default().render(), |m| m.render())?,
                Module::Git => self
                    .git
                    .as_ref()
                    .map_or(Git::default().render(), |m| m.render())?,

                Module::Custom(name) => self
                    .custom
                    .get(name)
                    .map_or(Ok(String::new()), |m| m.render())?,

                Module::Fill | Module::Symbol | Module::Cursor => continue,
            };
            prompt.push_str(&output);
        }

        // 2. Handle the Fill
        // pass the current prompt to Fill.
        if self.layout.contains(&Module::Fill) {
            let fill_rendered = self
                .fill
                .as_ref()
                .map(|f| f.expand(&prompt))
                .unwrap_or_else(|| Fill::default().expand(&prompt))
                .render()?;
            prompt.push_str(&fill_rendered);

            // We add the newline because Fill's job is to finish the line.
            prompt.push('\n');
        }

        // 3. Handle the Symbol
        if self.layout.contains(&Module::Symbol) {
            let symbol_rendered = self
                .symbol
                .as_ref()
                .unwrap_or(&Symbol::default())
                .render()?;
            prompt.push_str(&symbol_rendered);
        }

        Ok(prompt)
    }
}
pub fn run(config_file: String) -> Result<()> {
    // Read the entire TOML config file into a string
    let config_contents = fs::read_to_string(config_file)?;

    // Parse the string into a Prompt struct
    let config: Config = toml::from_str(&config_contents)?;

    // Validate user configs
    config.validate_user_config()?;

    // print!("{:#?} ", config);
    let prompt = Config::render(&config)?;

    println!("{prompt}");
    Ok(())
}

// ==== TESTS ====
#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a base config for testing
    fn create_test_config(layout: Vec<Module>) -> Config {
        Config {
            layout,
            username: None,
            pwd: None,
            fill: None,
            git: None,
            permission: None,
            symbol: None,
            custom: HashMap::new(),
        }
    }

    #[test]
    fn test_empty_layout_fails() {
        let config = create_test_config(vec![]);
        let result = config.validate_user_config();
        assert!(matches!(result, Err(Errors::EmptyLayout)));
    }

    #[test]
    fn test_duplicates() {
        let layout = vec![
            Module::Fill,
            Module::Fill,
            Module::Symbol,
            Module::Symbol,
            Module::Cursor,
            Module::Cursor,
        ];
        let config = create_test_config(layout);
        let result = config.validate_user_config();
        assert!(matches!(result, Err(Errors::DuplicateModule(_))))
    }

    #[test]
    fn test_undefined_custom_module_fails() {
        let config = create_test_config(vec![Module::Custom("missing".to_string())]);
        let result = config.validate_user_config();
        assert!(matches!(result, Err(Errors::ModuleNotDefined(_))));
    }
}
