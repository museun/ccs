use std::{fs::Metadata, path::PathBuf};

mod args;
pub use args::{Args, Mode, Tool};

mod command;
pub use command::{Command, Extra, Features, Options, Target, Toolchain};

mod parse;
pub use parse::{gather_reasons, gather_reasons_by, span, Code, Level, Message, Reason};

mod theme;
pub use theme::Theme;

mod options;
pub use options::{Filter, Filters, IncludeNotes, RenderOptions, RenderStyle};

mod config;
pub use config::Config;

pub mod render;

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

pub fn try_find_manifest(path: &mut PathBuf) -> anyhow::Result<()> {
    match path.components().last() {
        Some(s) if s.as_os_str() == "Cargo.toml" => {}
        Some(..) => {
            anyhow::ensure!(path.is_dir(), "a non-manifest file was provided");
            let tmp = path.join("Cargo.toml");
            anyhow::ensure!(
                std::fs::metadata(&tmp)
                    .ok()
                    .filter(Metadata::is_file)
                    .is_some(),
                "tried to find a Cargo.toml but couldn't find one"
            );
            *path = tmp;
        }
        _ => anyhow::bail!("you must provide the path to the manifest file (Cargo.toml)"),
    }
    Ok(())
}

pub fn is_nightly_available() -> bool {
    find_cargo(Toolchain::Nightly).is_some()
}
