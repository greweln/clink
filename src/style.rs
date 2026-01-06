use crate::{ANSI_RESET, IGNORE_END, IGNORE_START};
use serde::Deserialize;
// rename = serde deserializes the color string from user config in enum variant.
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    #[default]
    Foreground,
    Black,
    Grey,
    Gray,
    Red,
    RedBright,
    Green,
    GreenBright,
    Yellow,
    YellowBright,
    Blue,
    BlueBright,
    Cyan,
    CyanBright,
    Magenta,
    MagentaBright,
    White,
    WhiteBright,
}

impl Color {
    ///   ANSI escape code:  used to change the color of text in terminal output.
    ///   \x1b => is the escape character, which signals the start of an ANSI escape sequence.
    ///   [    => begins the Control Sequence Introducer (CSI).
    ///   31m  => sets the text color to red.

    // Convert enum variant to ANSI escape code
    fn to_ansi(&self) -> &'static str {
        match self {
            Color::Foreground => "\x1b[39m",
            Color::Black => "\x1b[30m",
            Color::Grey | Color::Gray => "\x1b[90m", // Bright Black
            Color::Red => "\x1b[31m",
            Color::RedBright => "\x1b[91m",
            Color::Green => "\x1b[32m",
            Color::GreenBright => "\x1b[92m",
            Color::Yellow => "\x1b[33m",
            Color::YellowBright => "\x1b[93m",
            Color::Blue => "\x1b[34m",
            Color::BlueBright => "\x1b[94m",
            Color::Magenta => "\x1b[35m",
            Color::MagentaBright => "\x1b[95m",
            Color::Cyan => "\x1b[36m",
            Color::CyanBright => "\x1b[96m",
            Color::White => "\x1b[37m",
            Color::WhiteBright => "\x1b[97m",
        }
    }

    /// Wraps text in the chosen color and resets it afterward.
    /// It wraps the invisible ANSI bytes in special markers so the shell
    /// ignores them when calculating prompt length and cursor position.
    pub fn apply(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }

        format!(
            "{IGNORE_START}{ansi}{IGNORE_END}{text}{IGNORE_START}{ANSI_RESET}{IGNORE_END}",
            ansi = self.to_ansi(),
            text = text
        )
    }
}
