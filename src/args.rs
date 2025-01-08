use std::{any::Any, path::PathBuf, str::FromStr};

use clap::{
    builder::{EnumValueParser, PossibleValue, ValueParser},
    Arg, ArgAction, ArgMatches, ValueEnum,
};

use crate::Filter;

#[derive(Copy, Clone, Default, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Tool {
    #[default]
    Clippy,
    Check,
}

impl ValueEnum for Tool {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Clippy, Self::Check]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Clippy => PossibleValue::new("clippy"),
            Self::Check => PossibleValue::new("check"),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum Mode {
    #[default]
    Report,
    Record,
    Replay,
    Watch,
}

impl ValueEnum for Mode {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Report, Self::Record, Self::Replay, Self::Watch]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Report => PossibleValue::new("report"),
            Self::Record => PossibleValue::new("record"),
            Self::Replay => PossibleValue::new("replay"),
            Self::Watch => PossibleValue::new("watch"),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GroupBy {
    Lint,
    File,
    #[default]
    Default,
}

impl ValueEnum for GroupBy {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::File, Self::Lint, Self::Default]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::File => PossibleValue::new("file"),
            Self::Lint => PossibleValue::new("lint"),
            Self::Default => PossibleValue::new("default"),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Sort {
    File,
    Lint,
    #[default]
    Default,
}

impl ValueEnum for Sort {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::File, Self::Lint, Self::Default]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::File => PossibleValue::new("file"),
            Self::Lint => PossibleValue::new("lint"),
            Self::Default => PossibleValue::new("default"),
        })
    }
}

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    pub tool: Tool,
    pub mode: Mode,
    pub sort: Sort,
    pub group_by: GroupBy,
    pub nightly: bool,
    pub explain: bool,
    pub include_notes: bool,
    pub tests: bool,
    pub examples: bool,
    pub path: Option<PathBuf>,
    pub annoying: bool,
    pub more_annoying: bool,
    pub filter: Vec<Filter>,
    pub warning: Vec<String>,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub target: Vec<String>,
    pub all_targets: bool,
    pub features: Vec<String>,
    pub all_features: bool,
    pub no_features: bool,
    pub delimiter: Option<String>,
    pub new_line: bool,
    pub ignore_config: bool,
    pub print_config_path: bool,
    pub print_default_config: bool,
    pub dry_run: bool,
}

impl Args {
    #[allow(clippy::too_many_lines)]
    pub fn parse() -> Self {
        let cmd = clap::Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .about(
                "simplifies the output of cargo clippy\n\n\
                this runs clippy (or check) and produces a more compact output",
            )
            .arg(
                Arg::new("tool")
                    .long("tool")
                    .action(ArgAction::Set)
                    .value_parser(EnumValueParser::<Tool>::new())
                    .ignore_case(true)
                    .default_value("clippy")
                    .help("specify the tool to use to check for lints"),
            )
            .arg(
                Arg::new("sort")
                    .short('s')
                    .long("sort")
                    .action(ArgAction::Set)
                    .value_parser(EnumValueParser::<Sort>::new())
                    .ignore_case(true)
                    .default_value("default")
                    .help_heading("sorting results")
                    .group("sorting")
                    .help("specifies how the results should be sorted"),
            )
            .arg(
                Arg::new("group_by")
                    .short('g')
                    .long("group")
                    .action(ArgAction::Set)
                    .value_parser(EnumValueParser::<GroupBy>::new())
                    .ignore_case(true)
                    .default_value("default")
                    .help_heading("sorting results")
                    .group("sorting")
                    .help("specifies how the results should be grouped"),
            )
            .arg(
                Arg::new("nightly")
                    .short('n')
                    .long("nightly")
                    .action(ArgAction::SetTrue)
                    .help("use the installed nightly version of clippy"),
            )
            .arg(
                Arg::new("tests")
                    .short('t')
                    .long("tests")
                    .action(ArgAction::SetTrue)
                    .help_heading("targets")
                    .help("checks only the test targets"),
            )
            .arg(
                Arg::new("examples")
                    .short('x')
                    .long("examples")
                    .action(ArgAction::SetTrue)
                    .help_heading("targets")
                    .help("checks only the example targets"),
            )
            .arg(
                Arg::new("target")
                    .long("target")
                    .help_heading("targets")
                    .action(ArgAction::Append)
                    .help("check a specific target"),
            )
            .arg(
                Arg::new("all_targets")
                    .long("all-targets")
                    .action(ArgAction::SetTrue)
                    .help_heading("targets")
                    .help("check all targets"),
            )
            .arg(
                Arg::new("features")
                    .long("features")
                    .short('F')
                    .help_heading("targets")
                    .action(ArgAction::Append)
                    .help("check a specific feature"),
            )
            .arg(
                Arg::new("all_features")
                    .long("all-features")
                    .action(ArgAction::SetTrue)
                    .help_heading("targets")
                    .help("check all features"),
            )
            .arg(
                Arg::new("no_default_features")
                    .long("no-default-features")
                    .action(ArgAction::SetTrue)
                    .help_heading("targets")
                    .help("disable all features"),
            )
            .arg(
                // TODO rename this to --manifest-path
                Arg::new("path")
                    .short('p')
                    .long("path")
                    .value_parser(clap::value_parser!(PathBuf))
                    .help("path to a specific Cargo.toml manifest"),
            )
            // TODO add -p / --pkgid like Cargo uses
            .arg(
                Arg::new("annoying")
                    .short('y')
                    .long("annoying")
                    .action(ArgAction::SetTrue)
                    .help_heading("controlling lints")
                    .help("use `clippy::all` and `clippy::nursery` (this requires nightly clippy)"),
            )
            .arg(
                Arg::new("more_annoying")
                    .short('Y')
                    .long("more-annoying")
                    .action(ArgAction::SetTrue)
                    .help_heading("controlling lints")
                    .help(
                        "use `clippy::all` and `clippy::nursery` and `clippy::pedantic` \
                        (this requires nightly clippy)",
                    ),
            )
            .arg(
                Arg::new("filter")
                    .short('f')
                    .long("filter")
                    .value_parser(ValueParser::new(Filter::from_str))
                    .action(ArgAction::Append)
                    .help_heading("controlling lints")
                    .help("ignore a specific warning or error")
                    .long_help(
                        "syntax: (warning|error)=(named_lint|all).\n\
                        example: -f error=all -f warning=unused_imports",
                    ),
            )
            .arg(
                Arg::new("warning")
                    .short('W')
                    .long("warning")
                    .value_name("lint")
                    .action(ArgAction::Append)
                    .help_heading("controlling lints")
                    .help("additional warning lints to use"),
            )
            .arg(
                Arg::new("allow")
                    .short('A')
                    .long("allow")
                    .value_name("lint")
                    .action(ArgAction::Append)
                    .help_heading("controlling lints")
                    .help("additional allow lints to use"),
            )
            .arg(
                Arg::new("deny")
                    .short('D')
                    .long("deny")
                    .value_name("lint")
                    .action(ArgAction::Append)
                    .help_heading("controlling lints")
                    .help("additional deny lints to use"),
            )
            .arg(
                Arg::new("explain")
                    .short('e')
                    .long("explain")
                    .action(ArgAction::SetTrue)
                    .help_heading("appearance")
                    .help("use the `explain` format")
                    .long_help("include a snippet of the code if available"),
            )
            .arg(
                Arg::new("include_notes")
                    .short('i')
                    .long("include")
                    .action(ArgAction::SetTrue)
                    .help_heading("appearance")
                    .help("include any `notes` if present")
                    .long_help(
                        "sometimes notes are provided to further explain a lint.\n\
                        these can be rather verbose. by default they are hidden,\n\
                        use this flag to show them",
                    ),
            )
            .arg(
                Arg::new("delimiter")
                    .short('d')
                    .long("delimiter")
                    .help_heading("appearance")
                    // .group("interspersed")
                    .help("append this delimited interspersed with each item"),
            )
            .arg(
                Arg::new("new_line")
                    .long("nl")
                    .action(ArgAction::SetTrue)
                    // .group("interspersed")
                    .help_heading("appearance")
                    .help("append a new line interspersed with each item"),
            )
            .arg(
                Arg::new("ignore_config")
                    .long("ignore-config")
                    .action(ArgAction::SetTrue)
                    .help_heading("meta")
                    .group("config")
                    .help("don't use the configuration file"),
            )
            .arg(
                Arg::new("print_config_path")
                    .long("print-config-path")
                    .action(ArgAction::SetTrue)
                    .group("config")
                    .exclusive(true)
                    .help_heading("meta")
                    .help("prints out the configuration path"),
            )
            .arg(
                Arg::new("print_default_config")
                    .long("print-default-config")
                    .action(ArgAction::SetTrue)
                    .group("config")
                    .exclusive(true)
                    .help_heading("meta")
                    .help("print out a default configuration"),
            )
            .arg(
                Arg::new("dry_run")
                    .long("dry-run")
                    .action(ArgAction::SetTrue)
                    .help_heading("meta")
                    .help("print out the command invocation -- don't actually run it"),
            )
            .arg(
                Arg::new("mode")
                    .long("mode")
                    .action(ArgAction::Set)
                    .value_parser(EnumValueParser::<Mode>::new())
                    .ignore_case(true)
                    .default_value("report")
                    .help("this changes the mode of the program")
                    .help_heading("development")
                    .long_help(
                        "- `watch` reruns the program on file change\n\
                        - `report` is the default 'one-shot' mode\n\n\
                        - `record` allows you to `replay` the output later\n\
                             this allows you to tweak things, or submit bug reports\n\
                             this'll produce a .ccs.json file in the `cwd`",
                    ),
            );

        let mut matches = cmd.get_matches();

        Self {
            tool: matches.remove_one("tool").unwrap_or_default(),
            mode: matches.remove_one("mode").unwrap_or_default(),
            sort: matches.remove_one("sort").unwrap_or_default(),
            group_by: matches.remove_one("group_by").unwrap_or_default(),
            nightly: matches.get_flag("nightly"),
            explain: matches.get_flag("explain"),
            include_notes: matches.get_flag("include_notes"),
            tests: matches.get_flag("tests"),
            examples: matches.get_flag("examples"),
            path: matches.remove_one("path"),
            annoying: matches.get_flag("annoying"),
            more_annoying: matches.get_flag("more_annoying"),
            filter: get_many(&mut matches, "filter"),
            warning: get_many(&mut matches, "warning"),
            allow: get_many(&mut matches, "allow"),
            deny: get_many(&mut matches, "deny"),
            target: get_many(&mut matches, "target"),
            all_targets: matches.get_flag("all_targets"),
            features: get_many(&mut matches, "features"),
            all_features: matches.get_flag("all_features"),
            no_features: matches.get_flag("no_default_features"),
            delimiter: matches.remove_one("delimiter"),
            new_line: matches.get_flag("new_line"),
            ignore_config: matches.get_flag("ignore_config"),
            print_config_path: matches.get_flag("print_config_path"),
            print_default_config: matches.get_flag("print_default_config"),
            dry_run: matches.get_flag("dry_run"),
        }
    }
}

fn get_many<T>(matches: &mut ArgMatches, key: &str) -> Vec<T>
where
    T: Any + Clone + Send + Sync + 'static,
{
    matches.remove_many(key).into_iter().flatten().collect()
}
