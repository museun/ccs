use std::{
    io::{stdout, BufRead},
    path::PathBuf,
};

use gumdrop::Options as _;

use ccs::{Args, Command, Extra, Extract, Long, Options, OutputKind, Short, Target, Toolchain};

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

fn is_nightly_available() -> bool {
    ccs::find_cargo(Toolchain::Nightly).is_some()
}

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

fn main() -> anyhow::Result<()> {
    let mut args = Args::parse_args_default_or_exit();

    if args.version {
        println!("{NAME}: {VERSION}");
        std::process::exit(0)
    }

    if args.nightly && !is_nightly_available() {
        eprintln!("rust nightly isn't installed");
        std::process::exit(1)
    }

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

    let target = match (args.tests, args.all_targets) {
        (.., true) => Target::All,
        (true, false) => Target::Test,
        (false, false) => Target::Normal,
    };

    let Args {
        allow,
        warning,
        deny,
        ..
    } = args;

    let opts = Options {
        format,
        toolchain,
        extra: Extra {
            allow,
            warning,
            deny,
        },
        path: args.path,
        target,
    };

    let child = command.build_command(opts)?;
    let mut w = stdout();

    let (mut a, mut b);
    let p: &mut dyn Extract = match format {
        OutputKind::Human => {
            a = Long::new();
            &mut a
        }
        OutputKind::Short => {
            b = Short::new();
            &mut b
        }
    };

    std::io::BufReader::new(child)
        .lines()
        .flatten()
        .try_for_each(|line| p.extract(&line, &mut w))
        .map_err(Into::into)
}
