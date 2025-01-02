use std::{fs::Metadata, path::PathBuf};

use ccs::{
    Args, Command, Config, Extra, Features, Filter, IncludeNotes, Mode, Options, RenderOptions,
    RenderStyle, Target, Theme, Tool, Toolchain,
};

mod interactive {
    use anstream::AutoStream;
    use ccs::{gather_reasons, Command, Options, Reason, RenderOptions, Theme};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use notify::Watcher;
    use owo_colors::OwoColorize;
    use std::{
        collections::BTreeMap,
        io::{BufReader, Write as _},
    };

    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            let mut out = std::io::stdout();
            _ = crossterm::execute!(
                &mut out,
                // crossterm::terminal::LeaveAlternateScreen,
                // crossterm::event::DisableMouseCapture,
                crossterm::cursor::Show
            );
            // _ = out.write_all(b"\x1b[?7h");
            _ = crossterm::terminal::disable_raw_mode();
        }
    }

    pub fn display(
        command: Command,
        opts: Options,
        theme: Theme,
        render_options: RenderOptions,
    ) -> anyhow::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        let mut out = std::io::stdout();

        crossterm::execute!(
            &mut out,
            // crossterm::terminal::EnterAlternateScreen,
            // crossterm::event::EnableMouseCapture,
            crossterm::cursor::Hide
        )?;

        // CSI Ps ; Ps r
        //   Set Scrolling Region [top;bottom] (default = full size of
        //   window) (DECSTBM), VT100.
        //
        // CSI ? Pm l
        //     Ps = 1 9  ⇒  Limit print to scrolling region (DECPEX),

        // CSI ? Pm h
        // Ps = 1 0 0 7  ⇒  Enable Alternate Scroll Mode, xterm.  This
        // Ps = 1 0 1 1  ⇒  Scroll to bottom on key press (rxvt).  This

        // out.write_all(b"\x1b[?7l")?;
        // out.flush()?;

        let _guard = Guard;

        let mut out = AutoStream::new(out, anstream::ColorChoice::Auto).lock();

        // TODO this should be customizable by the user
        const ALPHA: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

        let (tx, rx) = flume::unbounded();
        let mut notify = notify::recommended_watcher(move |ev| {
            if let Ok(ev) = ev {
                let _ = tx.send(ev);
            }
        })?;

        let path = match &opts.path {
            Some(path) => path.clone(),
            None => std::env::current_dir()?,
        };

        for path in ignore::Walk::new(path).flatten() {
            notify.watch(&path.into_path(), notify::RecursiveMode::NonRecursive)?;
        }

        let mut clipboard = arboard::Clipboard::new()?;
        let mut mapping = BTreeMap::new();

        loop {
            mapping.clear();

            let reasons = gather_reasons(BufReader::new(command.build_command(&opts)?));
            for (i, reason) in reasons
                .iter()
                .filter(|reason| {
                    if let Reason::CompilerMessage { message } = &reason {
                        !render_options
                            .is_ignored(message.level, message.code.as_ref().map(|c| &*c.code))
                    } else {
                        true
                    }
                })
                .enumerate()
            {
                if i > 0 {
                    if let Some(delim) =
                        &render_options.delimiter.as_ref().filter(|c| !c.is_empty())
                    {
                        writeln!(out, "{delim}")?;
                    } else if render_options.new_line {
                        writeln!(out)?;
                    }
                }

                // TODO don't loop here, double up on the characters
                let alpha = ALPHA[i % ALPHA.len()];
                mapping.insert(alpha, i);

                write!(
                    out,
                    "{} ",
                    std::str::from_utf8(std::slice::from_ref(&alpha))
                        .unwrap()
                        .color(owo_colors::Rgb(0xFF, 0x00, 0x00))
                )?;

                reason.render(&render_options, &theme, &mut out)?;
            }

            let mut count = 0;
            'try_it: loop {
                if !try_read_event(&reasons, &mapping, &mut clipboard, &mut out)? {
                    return Ok(());
                }

                match rx.try_recv() {
                    Err(flume::TryRecvError::Empty) if count == 0 => {}
                    Err(flume::TryRecvError::Empty) => break 'try_it,
                    Err(flume::TryRecvError::Disconnected) => return Ok(()),
                    Ok(..) => count += 1,
                }
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
        }
    }

    fn try_read_event(
        reasons: &[Reason],
        mapping: &BTreeMap<u8, usize>,
        clipboard: &mut arboard::Clipboard,
        out: &mut impl std::io::Write,
    ) -> anyhow::Result<bool> {
        if crossterm::event::poll(std::time::Duration::ZERO)? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Release {
                        let delta = match key_event.code {
                            KeyCode::Up => -1,
                            KeyCode::Down => 1,
                            KeyCode::Home => i16::MIN,
                            KeyCode::End => i16::MAX,
                            // TODO need terminal size for this
                            // KeyCode::PageUp => todo!(),
                            // KeyCode::PageDown => todo!(),
                            _ => {
                                return handle_key_press(
                                    key_event, &reasons, mapping, clipboard, out,
                                )
                            }
                        };

                        handle_scroll(out, delta)?;
                    }
                }
                crossterm::event::Event::Mouse(mouse_event) => {
                    let delta = match mouse_event.kind {
                        crossterm::event::MouseEventKind::ScrollDown => 3,
                        crossterm::event::MouseEventKind::ScrollUp => -3,
                        _ => return Ok(true),
                    };

                    handle_scroll(out, delta)?;
                }
                _ => (),
            }
        }
        Ok(true)
    }

    fn handle_scroll(_out: &mut impl std::io::Write, delta: i16) -> std::io::Result<()> {
        let _offset = delta.abs() as u16;
        match delta {
            delta if delta < 0 => {
                // crossterm::execute!(out, crossterm::terminal::ScrollDown(offset))
                Ok(())
            }
            delta if delta > 0 => {
                // crossterm::execute!(out, crossterm::terminal::ScrollUp(offset))
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn handle_key_press(
        key_event: KeyEvent,
        reasons: &[Reason],
        mapping: &BTreeMap<u8, usize>,
        clipboard: &mut arboard::Clipboard,
        out: &mut impl std::io::Write,
    ) -> anyhow::Result<bool> {
        if key_event.code == KeyCode::Char('c') && key_event.modifiers == KeyModifiers::CONTROL {
            return Ok(false);
        }

        if let KeyCode::Char(ch) = key_event.code {
            match ch {
                'c' if key_event.modifiers == KeyModifiers::CONTROL => return Ok(false),
                '?' => {
                    writeln!(out, "available links:")?;
                    for key in mapping.keys() {
                        let s = std::str::from_utf8(std::slice::from_ref(&key)).unwrap();
                        let link = build_link(mapping, reasons, *key).unwrap();
                        writeln!(out, "{s}: {link}")?;
                    }
                    out.flush()?;
                }
                selected => {
                    if let Some(link) = build_link(mapping, reasons, selected as u8) {
                        clipboard.set_text(&link)?;
                    }
                }
            }
        }

        Ok(true)
    }

    fn build_link(mapping: &BTreeMap<u8, usize>, reasons: &[Reason], index: u8) -> Option<String> {
        let index = mapping.get(&index)?;
        let Reason::CompilerMessage { message } = &reasons[*index] else {
            return None;
        };

        let span = message.spans.first()?;
        let link = format!(
            "{}:{}:{}",
            span.file_name, span.line_start, span.column_start
        );
        Some(link)
    }
}

mod report {
    use anstream::AutoStream;
    use ccs::{gather_reasons, Command, Options, Reason, RenderOptions, Theme};
    use std::io::{BufReader, Write as _};

    pub fn display(
        command: Command,
        opts: Options,
        theme: Theme,
        render_options: RenderOptions,
    ) -> anyhow::Result<()> {
        let mut out = AutoStream::new(std::io::stdout(), anstream::ColorChoice::Auto).lock();

        gather_reasons(BufReader::new(command.build_command(&opts)?))
            .into_iter()
            .filter(|reason| {
                if let Reason::CompilerMessage { message } = &reason {
                    !render_options
                        .is_ignored(message.level, message.code.as_ref().map(|c| &*c.code))
                } else {
                    true
                }
            })
            .enumerate()
            .try_for_each(|(i, reason)| {
                if i > 0 {
                    if let Some(delim) =
                        &render_options.delimiter.as_ref().filter(|c| !c.is_empty())
                    {
                        writeln!(out, "{delim}")?;
                    } else if render_options.new_line {
                        writeln!(out)?;
                    }
                }
                reason.render(&render_options, &theme, &mut out)?;
                std::io::Result::Ok(())
            })
            .map_err(Into::into)
    }
}

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

        continuation,
        delimiter: args.delimiter,
        new_line: args.new_line,

        ..RenderOptions::default()
    };

    for filter in std::mem::take(&mut args.filter) {
        render_options = match filter {
            Filter::AllWarnings => render_options.without_warnings(),
            Filter::AllErrors => render_options.without_errors(),
            Filter::Error(lint) => render_options.without_error(lint),
            Filter::Warning(lint) => render_options.without_warning(lint),
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

    match args.mode {
        Mode::Interactive => interactive::display(command, opts, theme, render_options),
        Mode::Report => report::display(command, opts, theme, render_options),
    }
}
