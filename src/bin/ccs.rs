use ccs::{
    render, Args, Command, Config, Extra, Features, Filter, Filters, IncludeNotes, Mode, Options,
    RenderOptions, RenderStyle, Target, Theme, Tool, Toolchain,
};

// TODO decide if we should replace \ with / on windows

#[allow(clippy::too_many_lines)]
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
        let config = toml::to_string_pretty(&Config::default());
        let config = config.expect("valid default configuration");
        println!("{config}");
        std::process::exit(0)
    }

    let mut theme = Theme::default();
    let mut continuation = None;

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

            if config
                .continuation
                .as_ref()
                .filter(|c| !c.is_empty())
                .is_some()
            {
                continuation = config.continuation;
            }

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

    if args.nightly && !ccs::is_nightly_available() {
        eprintln!("rust nightly isn't installed");
        std::process::exit(1)
    }

    if let Some(path) = args.path.as_mut() {
        ccs::try_find_manifest(path)?;
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

    let render = RenderOptions {
        render: args
            .explain
            .then_some(RenderStyle::Full)
            .unwrap_or_default(),

        include_notes: args
            .include_notes
            .then_some(IncludeNotes::Yes)
            .unwrap_or_default(),

        continuation,
        delimiter: args.delimiter,
        new_line: args.new_line,
    };

    let mut filters = Filters::default();
    for filter in std::mem::take(&mut args.filter) {
        filters = match filter {
            Filter::AllWarnings => filters.without_warnings(),
            Filter::AllErrors => filters.without_errors(),
            Filter::Error(lint) => filters.without_error(lint),
            Filter::Warning(lint) => filters.without_warning(lint),
        }
    }

    let Args {
        allow,
        warning,
        deny,
        dry_run,
        tool,
        sort,
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
        sort,
        filters,
    };

    match args.mode {
        Mode::Report => render::display(args.group_by, command, opts, theme, render),
        Mode::Record => render::record(command, opts),
        Mode::Replay => render::replay(args.group_by, opts, render, theme),
        Mode::Watch => render::watch(args.group_by, command, opts, theme, render),
    }
}
