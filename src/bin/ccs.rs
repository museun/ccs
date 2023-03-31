use std::{fs::Metadata, io::BufReader, path::PathBuf};

use anstream::AutoStream;

use ccs::{
    gather_reasons, Args, Command, Config, Extra, Features, IncludeNotes, Options, Render, Target,
    Theme, Toolchain,
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

fn is_nightly_available() -> bool {
    ccs::find_cargo(Toolchain::Nightly).is_some()
}

fn main() -> anyhow::Result<()> {
    let mut args = ccs::args().run();

    if args.print_config_path {
        match Config::get_config_path() {
            Some(path) => {
                println!("{}", path.to_string_lossy());
                std::process::exit(0)
            }
            None => {
                eprintln!("cannot locate a configuration directory");
                std::process::exit(1)
            }
        }
    }

    if args.print_default_config {
        let config = Config::default();
        println!(
            "{s}",
            s = toml::to_string_pretty(&config) //
                .expect("valid default configuration")
        );
        std::process::exit(0)
    }

    let mut theme = Theme::default();

    if let Some(path) = Config::get_config_path() {
        let mut config = match Config::load(&path) {
            Some(Ok(config)) => config,
            Some(Err(err)) => {
                eprintln!("cannot parse configuration file: {err}");
                std::process::exit(1)
            }
            None => {
                let dir = path.parent().expect("configuration directory");
                let _ = std::fs::create_dir_all(dir);
                if let Err(err) = Config::default().save(&path) {
                    eprintln!("cannot write default config: {err}");
                    std::process::exit(1)
                }
                Config::load(&path)
                    .transpose()
                    .ok()
                    .flatten()
                    .expect("default config should be valid")
            }
        };

        args.warning.append(&mut config.lints.warn);
        args.allow.append(&mut config.lints.allow);
        args.deny.append(&mut config.lints.deny);

        args.nightly ^= config.options.nightly;
        args.explain ^= config.options.explain;
        args.new_line ^= config.options.new_line;
        args.include_notes ^= config.options.include_notes;

        args.delimiter.get_or_insert(config.options.delimiter);

        theme = config.theme;
    }

    if args.nightly && !is_nightly_available() {
        eprintln!("rust nightly isn't installed");
        std::process::exit(1)
    }

    if let Some(path) = args.path.as_mut() {
        try_find_manifest(path)?;
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

    let mut target = match (args.tests, args.all_targets) {
        (.., true) => Target::All,
        (true, false) => Target::Test,
        (false, false) => Target::Default,
    };

    if !args.target.is_empty() {
        target = Target::Specific(std::mem::take(&mut args.target));
    }

    let features = match (args.all_features, &*args.feature) {
        (true, ..) => Features::All,
        (false, []) => Features::Default,
        _ => Features::Specific(std::mem::take(&mut args.feature)),
    };

    let render = args.explain.then_some(Render::Full).unwrap_or_default();
    let include_notes = args
        .include_notes
        .then_some(IncludeNotes::Yes)
        .unwrap_or_default();

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

    let reasons = gather_reasons(BufReader::new(child));
    let len = reasons.len();

    reasons
        .into_iter()
        .enumerate()
        .try_for_each(|(i, reason)| {
            use std::io::Write as _;
            reason.render(render, include_notes, &theme, &mut out)?;
            if i < len.saturating_sub(1) {
                if let Some(delim) = &args.delimiter.as_ref().filter(|c| !c.is_empty()) {
                    writeln!(out, "{delim}")?;
                }
                if args.new_line {
                    writeln!(out)?;
                }
            }
            std::io::Result::Ok(())
        })
        .map_err(Into::into)
}
