# a simplier output for `cargo clippy`

```
Usage: ccs [OPTIONS]

simplifies the output of cargo clippy

this runs clippy and produces are smaller output
with the `-e` flag, it'll also try to provide some context

Optional arguments:
  -h, --help             prints the help message
  -v, --version          prints the current version of this tool
  -n, --nightly          use the installed nightly version of clippy
  -e, --explain          use the `explain` format
  -t, --tests            check only test targets
  -p, --path PATH        path to a specific Cargo.toml manifest. this defaults to the `cwd`
  -y, --annoying         use `clippy::all` and `clippy::nursery` (requires nightly clippy)
  -Y, --more-annoying    use `clippy::all` and `clippy::pedantic`
  -W, --warn WARNING     additional warning lints to use
  -A, --allow ALLOW      additional allow lints to use
  -D, --deny DENY        additional deny lints to use
  --target TARGET        check a specific target
  --all-targets          check all targets
  --feature FEATURE      check a specific feature
  --all-features         check all features
  --delimiter DELIMITER  append this delimited interpersed with each item
  --nl                   append a new line interpersed with each item
  --dry-run              print out the command invocation -- don't actually run it
```

example:

> ccs

```
warning adding items after statements is confusing, since items exist from the start of the scope
 ⮡ src\parse.rs:165:9 (clippy::items_after_statements)
warning more than 3 bools in a struct
 ⮡ src\args.rs:8:1 (clippy::struct_excessive_bools)
warning this method could have a `#[must_use]` attribute
 ⮡ src\command.rs:15:5 (clippy::must_use_candidate)
warning this method could have a `#[must_use]` attribute
 ⮡ src\command.rs:26:5 (clippy::must_use_candidate)
warning this method could have a `#[must_use]` attribute
 ⮡ src\command.rs:37:5 (clippy::must_use_candidate)
warning docs for function returning `Result` missing `# Errors` section
 ⮡ src\command.rs:41:5 (clippy::missing_errors_doc)
```

vs

> cargo clippy --message-format=short

```
src\parse.rs:165:9: warning: adding items after statements is confusing, since items exist from the st
art of the scope
src\args.rs:8:1: warning: more than 3 bools in a struct
src\command.rs:15:5: warning: this method could have a `#[must_use]` attribute
src\command.rs:26:5: warning: this method could have a `#[must_use]` attribute
src\command.rs:37:5: warning: this method could have a `#[must_use]` attribute
src\command.rs:41:5: warning: docs for function returning `Result` missing `# Errors` section
```
