mod args;
pub use args::{args, Args};

mod command;
pub use command::{Command, Extra, Features, Options, Target, Toolchain};

mod parse;
pub use parse::{gather_reasons, Reason};

pub fn find_cargo(toolchain: Toolchain) -> Option<String> {
    let mut cmd = std::process::Command::new("rustup");
    if let Some(toolchain) = toolchain.as_str() {
        cmd.arg(toolchain);
    }

    let mut output = cmd
        .args(["which", "cargo"])
        .output()
        .ok()
        .map(|c| String::from_utf8(c.stdout))?
        .ok()?;

    output.drain(output.trim_end().len()..);
    Some(output)
}

mod theme;
pub use theme::Theme;

mod render;
pub use render::{IncludeNotes, Render};

mod config;
pub use config::Config;
