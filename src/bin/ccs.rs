use std::{io::BufReader, path::PathBuf};

use anstream::AutoStream;
use gumdrop::Options as _;

use ccs::{
    gather_reasons, Args, Command, Extra, Features, Options, Render, Target, Theme, Toolchain,
};

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

    if let Some(path) = args.path.as_mut() {
        try_find_manifest(path)?
    }

    let mut toolchain = args
        .nightly
        .then_some(Toolchain::Nightly)
        .unwrap_or_default();

    let command = if args.nightly && args.annoying {
        toolchain = Toolchain::Nightly;
        Command::annoying()
    } else if args.more_annoying {
        Command::more_annoying()
    } else {
        Command::clippy()
    };

    let render = args.explain.then_some(Render::Full).unwrap_or_default();

    let mut target = match (args.tests, args.all_targets) {
        (.., true) => Target::All,
        (true, false) => Target::Test,
        (false, false) => Target::Default,
    };

    if !args.target.is_empty() {
        target = Target::Specific(std::mem::take(&mut args.target))
    }

    let features = match (args.all_features, &*args.feature) {
        (true, ..) => Features::All,
        (false, []) => Features::Default,
        _ => Features::Specific(std::mem::take(&mut args.feature)),
    };

    let Args {
        allow,
        warning,
        deny,
        dry_run,
        ..
    } = args;

    let opts = Options {
        toolchain,
        extra: Extra {
            allow,
            warning,
            deny,
        },
        path: args.path,
        target,
        features,
        dry_run,
    };

    let child = command.build_command(opts)?;

    let mut out = AutoStream::new(std::io::stdout(), anstream::ColorChoice::Auto).lock();
    let theme = Theme::default();

    let reasons = gather_reasons(BufReader::new(child));
    let len = reasons.len();

    reasons
        .into_iter()
        .enumerate()
        .try_for_each(|(i, reason)| {
            use std::io::Write as _;
            reason.render(render, &theme, &mut out)?;
            if i < len.saturating_sub(1) {
                if let Some(delim) = &args.delimiter {
                    writeln!(out, "{delim}")?;
                } else if args.new_line {
                    writeln!(out)?;
                }
            }
            std::io::Result::Ok(())
        })
        .map_err(Into::into)
}
