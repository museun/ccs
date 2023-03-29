use std::path::PathBuf;

#[derive(gumdrop::Options, Debug)]
/// simplifies the output of cargo clippy
///
/// this runs clippy and produces are smaller output
/// with the `-e` flag, it'll also try to provide some context
pub struct Args {
    /// prints the help message
    #[options(help_flag)]
    pub help: bool,

    /// prints the current version of this tool
    pub version: bool,

    /// use the installed nightly version of clippy
    pub nightly: bool,

    /// use the `explain` format
    pub explain: bool,

    /// check only test targets
    pub tests: bool,

    /// path to a specific Cargo.toml manifest. this defaults to the `cwd`
    pub path: Option<PathBuf>,

    /// use `clippy::all` and `clippy::nursery` (requires nightly clippy)
    #[options(short = "y")]
    pub annoying: bool,

    /// use `clippy::all` and `clippy::pedantic`
    #[options(short = "Y")]
    pub more_annoying: bool,

    /// additional warning lints to use
    #[options(short = "W", long = "warn")]
    pub warning: Vec<String>,

    /// additional allow lints to use
    #[options(short = "A", long = "allow")]
    pub allow: Vec<String>,

    /// additional deny lints to use
    #[options(short = "D", long = "deny")]
    pub deny: Vec<String>,

    /// check a specific target
    #[options(no_short)]
    pub target: Vec<String>,

    /// check all targets
    #[options(no_short)]
    pub all_targets: bool,

    /// check a specific feature
    #[options(no_short)]
    pub feature: Vec<String>,

    /// check all features
    #[options(no_short)]
    pub all_features: bool,

    /// append this delimited interpersed with each item
    #[options(no_short)]
    pub delimiter: Option<String>,

    /// append a new line interpersed with each item
    #[options(no_short, long = "nl")]
    pub new_line: bool,

    /// print out the command invocation -- don't actually run it
    #[options(no_short)]
    pub dry_run: bool,
}
