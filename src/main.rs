// TODO: envs in bashrc of where is best to store them?

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Prompt {
    cwd: Option<bool>,
    username: Option<bool>,
    git: Option<bool>,
    dashes: Option<bool>,
    // TODO: dash : Option<String>
}

impl Prompt {
    fn new(&self) -> String {
        let mut prompt = String::new();

        if self.cwd == Some(true) {
            prompt.push_str("cwd");
        }
        if self.username == Some(true) {
            prompt.push_str("username");
        }
        if self.git == Some(true) {
            prompt.push_str("git");
        }
        if self.dashes == Some(true) {
            prompt.push_str("dashes");
        }
        prompt
    }

    fn run(&self) {
        if self.new().is_empty() {
            print!("$")
        } else {
            println!("{}", self.new())
        }
    }
}

fn main() {
    // get the struct fields from env variables, create a prompt with them and print it
    match envy::prefixed("CLINK_").from_env::<Prompt>() {
        Ok(prompt) => prompt.run(),
        Err(err) => {
            panic!("{}", err);
        }
    };
}
