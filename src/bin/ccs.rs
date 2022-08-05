use std::{
    io::{stdout, BufRead},
    path::PathBuf,
};

use gumdrop::Options as _;

use ccs::{Args, Command, LongParser, Options, OutputKind, Parse, ShortParser, Toolchain};

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

    let mut toolchain = args
        .nightly
        .then_some(Toolchain::Nightly)
        .unwrap_or_default();

    let command = if args.nightly {
        toolchain = Toolchain::Nightly;
        Command::annoying()
    } else {
        Command::clippy()
    };

    let format = args
        .explain
        .then_some(OutputKind::Human)
        .unwrap_or_default();

    let opts = Options {
        format,
        toolchain,
        extra: args.additional,
        path: args.path,
        tests: args.tests,
    };

    let child = command.build_command(opts)?;

    let mut w = stdout();

    let (mut a, mut b);
    let p: &mut dyn Parse = match format {
        OutputKind::Human => {
            a = LongParser::new();
            &mut a
        }
        OutputKind::Short => {
            b = ShortParser::new();
            &mut b
        }
    };

    for line in std::io::BufReader::new(child).lines().flatten() {
        p.extract(&line, &mut w)?
    }
    Ok(())
}
