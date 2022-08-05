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

    /// use the `explain` format
    pub explain: bool,

    /// check only test targets
    pub tests: bool,

    /// check all targets
    pub all_targets: bool,

    /// path to a specific Cargo.toml manifest. this defaults to the `cwd`
    #[options(meta = "<path>")]
    pub path: Option<PathBuf>,

    /// additional warning lints to use
    #[options(short = "W", long = "warn", meta = "<string>")]
    pub warning: Vec<String>,

    /// additional allow lints to use
    #[options(short = "A", long = "allow", meta = "<string>")]
    pub allow: Vec<String>,

    /// additional deny lints to use
    #[options(short = "D", long = "deny", meta = "<string>")]
    pub deny: Vec<String>,

    /// use `clippy::all` and `clippy::nursery` (and nightly clippy)
    #[options(short = "y", default = "false")]
    pub annoying: bool,

    /// use nightly
    #[options(default = "false")]
    pub nightly: bool,
}
