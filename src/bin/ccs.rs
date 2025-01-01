use std::{fs::Metadata, io::BufReader, path::PathBuf};

use anstream::AutoStream;

use ccs::{
    gather_reasons, Args, Command, Config, Extra, Features, IncludeNotes, Options, Reason,
    RenderOptions, RenderStyle, Target, Theme, Tool, Toolchain,
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
    let mut args = Args::parse();

    if matches!(args.tool, Tool::Check) && (args.annoying || args.more_annoying) {
        eprintln!("Error: -y / -Y requires `--tool clippy`");
        std::process::exit(1)
    }

    if args.all_features && args.no_features {
        eprintln!("Error: `all-features` and `no-default-features` are exclusive");
        std::process::exit(1)
    }

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

    let mut continuation = Some(Config::CONTINUATION);

    if !args.ignore_config {
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

            continuation = config.continuation;

            args.warning.append(&mut config.lints.warn);
            args.allow.append(&mut config.lints.allow);
            args.deny.append(&mut config.lints.deny);

            // args.tool = config.tool;

            args.nightly ^= config.options.nightly;
            args.explain ^= config.options.explain;
            args.new_line ^= config.options.new_line;
            args.include_notes ^= config.options.include_notes;

            args.delimiter.get_or_insert(config.options.delimiter);

            theme = config.theme;
        }
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
        toolchain = Toolchain::Nightly;
        Command::more_annoying()
    } else {
        Command::default_lints()
    };

    let mut target = match (args.tests, args.examples, args.all_targets) {
        (.., true) => Target::All,
        (true, _, false) => Target::Test,
        (_, true, false) => Target::Example,
        (false, false, false) => Target::Default,
    };

    if !args.target.is_empty() {
        target = Target::Specific(std::mem::take(&mut args.target));
    }

    let features = match (args.all_features, args.no_features, &*args.features) {
        (true, false, ..) => Features::All,
        (false, true, ..) => Features::None,
        (false, false, []) => Features::Default,
        _ => Features::Specific(std::mem::take(&mut args.features)),
    };

    let mut render_options = RenderOptions {
        render: args
            .explain
            .then_some(RenderStyle::Full)
            .unwrap_or_default(),

        include_notes: args
            .include_notes
            .then_some(IncludeNotes::Yes)
            .unwrap_or_default(),

        ..RenderOptions::default()
    };

    for filter in std::mem::take(&mut args.filter) {
        render_options = match filter {
            ccs::Filter::AllWarnings => render_options.without_warnings(),
            ccs::Filter::AllErrors => render_options.without_errors(),
            ccs::Filter::Error(lint) => render_options.without_error(lint),
            ccs::Filter::Warning(lint) => render_options.without_warning(lint),
        }
    }

    let Args {
        allow,
        warning,
        deny,
        dry_run,
        tool,
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
        tool,
    };

    let reasons = gather_reasons(BufReader::new(command.build_command(opts)?));
    let mut out = AutoStream::new(std::io::stdout(), anstream::ColorChoice::Auto).lock();

    reasons
        .into_iter()
        .filter(|reason| {
            if let Reason::CompilerMessage { message } = &reason {
                !render_options.is_ignored(message.level, message.code.as_ref().map(|c| &*c.code))
            } else {
                true
            }
        })
        .enumerate()
        .try_for_each(|(i, reason)| {
            use std::io::Write as _;
            if i > 0 {
                if let Some(delim) = &args.delimiter.as_ref().filter(|c| !c.is_empty()) {
                    writeln!(out, "{delim}")?;
                } else if args.new_line {
                    writeln!(out)?;
                }
            }
            reason.render(&render_options, &theme, &continuation, &mut out)?;
            std::io::Result::Ok(())
        })
        .map_err(Into::into)
}
