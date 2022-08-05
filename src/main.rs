use std::{
    io::{stdout, BufRead},
    path::PathBuf,
};

use gumdrop::Options;
use regex::Regex;

mod args;
use args::Args;

use crate::write::WriteExt;

mod command;
mod parse;
mod write;

fn try_find_manifest(path: &mut PathBuf) -> anyhow::Result<()> {
    match path.components().last() {
        Some(s) if s.as_os_str() == "Cargo.toml" => {}
        Some(..) => {
            anyhow::ensure!(path.is_dir(), "a non-manifest file was provided");
            let tmp = path.join("Cargo.toml");
            anyhow::ensure!(
                std::fs::metadata(&tmp)
                    .ok()
                    .filter(|c| c.is_file())
                    .is_some(),
                "tried to find a Cargo.toml but couldn't find one"
            );
            *path = tmp
        }
        _ => anyhow::bail!("you must provide the path to the manifest file (Cargo.toml)"),
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse_args_default_or_exit();

    // TODO disable colors via flag
    if std::env::var("NO_COLOR").is_ok() {
        yansi::Paint::disable()
    }

    if let Some(path) = args.path.as_mut() {
        try_find_manifest(path)?
    }

    const PATTERN: &str = r#"(?m)(?P<path>^.*?:\d{1,}:\d{1,}):\s(?P<kind>(error\[(?P<code>E\d{1,})\]|warning)):\s(?P<message>.*?)$"#;
    let pattern = Regex::new(PATTERN).unwrap();

    let command = args
        .nightly
        .then(command::Command::annoying)
        .unwrap_or_else(command::Command::clippy);

    let toolchain = args
        .nightly
        .then_some(command::Toolchain::Nightly)
        .unwrap_or_default();

    let child = command.build_command(args.additional, args.path, args.tests, toolchain)?;

    let mut w = stdout();
    let mut state = write::State::new(&pattern, args.line_breaks);

    std::io::BufReader::new(child)
        .lines()
        .flatten()
        .try_for_each(|line| w.format_line(&line, &mut state))
        .map_err(Into::into)
}
