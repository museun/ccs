use std::path::PathBuf;

use bpaf::Bpaf;

#[derive(Bpaf, Debug)]
/// simplifies the output of cargo clippy
///
/// this runs clippy and produces are smaller output
/// with the `-e` flag, it'll also try to provide some context
#[bpaf(options, version)]
pub struct Args {
    /// use the installed nightly version of clippy
    #[bpaf(short('n'), long("nightly"))]
    pub nightly: bool,

    /// use the `explain` format
    #[bpaf(short('e'), long("explain"))]
    pub explain: bool,

    /// include any `notes` if present
    #[bpaf(short('i'), long("include"))]
    pub include_notes: bool,

    /// check only test targets
    #[bpaf(short, long)]
    pub tests: bool,

    /// path to a specific Cargo.toml manifest. this defaults to the `cwd`
    #[bpaf(short, long)]
    pub path: Option<PathBuf>,

    /// use `clippy::all` and `clippy::nursery` (requires nightly clippy)
    #[bpaf(short('y'), long)]
    pub annoying: bool,

    /// use `clippy::all` and `clippy::pedantic`
    #[bpaf(short('Y'), long)]
    pub more_annoying: bool,

    /// additional warning lints to use
    #[bpaf(short('W'), long)]
    pub warning: Vec<String>,

    /// additional allow lints to use
    #[bpaf(short('A'), long)]
    pub allow: Vec<String>,

    /// additional deny lints to use
    #[bpaf(short('D'), long)]
    pub deny: Vec<String>,

    /// check a specific target
    pub target: Vec<String>,

    /// check all targets
    pub all_targets: bool,

    /// check a specific feature
    pub feature: Vec<String>,

    /// check all features
    pub all_features: bool,

    /// append this delimited interpersed with each item
    #[bpaf(long("delim"))]
    pub delimiter: Option<String>,

    /// append a new line interpersed with each item
    #[bpaf(long("nl"))]
    pub new_line: bool,

    /// prints out the configuration path
    #[bpaf(long("config-path"))]
    pub print_config_path: bool,

    /// print out a default configuration
    #[bpaf(long("default-config"))]
    pub print_default_config: bool,

    /// print out the command invocation -- don't actually run it
    pub dry_run: bool,
}
