use std::io::BufReader;

use anstream::AutoStream;
use report::Renderer;

use crate::{
    args::GroupBy, gather_reasons_by, parse::sort_reasons_by, Command, Options, Reason,
    RenderOptions, Theme,
};

mod report;

pub fn display(
    group_by: GroupBy,
    command: Command<'_>,
    opts: Options,
    theme: Theme,
    render_options: RenderOptions,
) -> anyhow::Result<()> {
    let mut out = AutoStream::new(std::io::stdout(), anstream::ColorChoice::Auto).lock();
    let command = BufReader::new(command.build_command(&opts)?);

    let mut reasons = gather_reasons_by(command, &opts.filters);
    sort_reasons_by(&mut reasons, opts.sort);

    let renderer = Renderer::new(&render_options, theme, &mut out);
    match group_by {
        GroupBy::Lint => renderer.group_by_lint().render(&reasons),
        GroupBy::File => renderer.group_by_file().render(&reasons),
        GroupBy::Default => renderer.group_by_default().render(&reasons),
    }?;

    Ok(())
}

static RECORD_FILE: &str = ".ccs.json";

pub fn record(command: Command<'_>, opts: Options) -> anyhow::Result<()> {
    let command = BufReader::new(command.build_command(&opts)?);

    std::fs::write(
        RECORD_FILE,
        serde_json::to_string_pretty(&gather_reasons_by(command, &opts.filters))?,
    )?;

    Ok(())
}

pub fn replay(
    group_by: GroupBy,
    opts: Options,
    render_options: RenderOptions,
    theme: Theme,
) -> anyhow::Result<()> {
    use anyhow::Context as _;
    let data = std::fs::read(RECORD_FILE).with_context(|| {
        anyhow::anyhow!("cannot find {RECORD_FILE}. use `ccs --mode record` to record a session")
    })?;

    let mut reasons: Vec<Reason> = serde_json::from_slice(&data)?;
    sort_reasons_by(&mut reasons, opts.sort);

    let mut out = AutoStream::new(std::io::stdout(), anstream::ColorChoice::Auto).lock();
    let renderer = Renderer::new(&render_options, theme, &mut out);
    match group_by {
        GroupBy::Lint => renderer.group_by_lint().render(&reasons),
        GroupBy::File => renderer.group_by_file().render(&reasons),
        GroupBy::Default => renderer.group_by_default().render(&reasons),
    }?;

    Ok(())
}
