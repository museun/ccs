use owo_colors::{AnsiColors, DynColors};

pub struct Theme {
    pub warning: DynColors,
    pub error: DynColors,
    pub unknown: DynColors,
    pub highlight: DynColors,
    pub lint_name: DynColors,
    pub location: DynColors,
    pub message: DynColors,
    pub continuation: DynColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            warning: DynColors::Ansi(AnsiColors::BrightYellow),
            error: DynColors::Ansi(AnsiColors::BrightRed),
            unknown: DynColors::Ansi(AnsiColors::Cyan),
            highlight: DynColors::Ansi(AnsiColors::BrightBlue),
            lint_name: DynColors::Ansi(AnsiColors::Magenta),
            location: DynColors::Ansi(AnsiColors::Green),
            message: DynColors::Ansi(AnsiColors::BrightWhite),
            continuation: DynColors::Ansi(AnsiColors::BrightBlack),
        }
    }
}

// TODO read from a configuration file
