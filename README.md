# a simplier output for `cargo clippy`

```
simplifies the output of cargo clippy

this runs clippy and produces are smaller output

Usage: ccs [OPTIONS]

Options:
  -n, --nightly
          use the installed nightly version of clippy

  -p, --path <path>
          path to a specific Cargo.toml manifest

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

targets:
  -t, --tests
          check only the test target

      --target <target>
          check a specific target

      --all-targets
          check all targets

      --feature <feature>
          check a specific feature

      --all-features
          check all features

controlling lints:
  -y, --annoying
          use `clippy::all` and `clippy::nursery` (this requires nightly clippy)

  -Y, --more-annoying
          use `clippy::all` and `clippy::nursery` and `clippy::pedantic` (this requires nightly clippy)

  -f, --filter <filter>
          syntax: (warning|error)=(named_lint|all).
          example: -f error=all -f warning=unused_imports

  -W, --warning <lint>
          additional warning lints to use

  -A, --allow <lint>
          additional allow lints to use

  -D, --deny <lint>
          additional deny lints to use

appearance:
  -e, --explain
          include a snippet of the code if available

  -i, --include
          sometimes notes are provided to further explain a lint.
          these can be rather verbose. by default they are hidden,
          use this flag to show them

      --delimiter <delimiter>
          append this delimited interspersed with each item

      --nl
          append a new line interspersed with each item

meta:
      --ignore-config
          don't use the configuration file

      --print-config-path
          prints out the configuration path

      --print-default-config
          print out a default configuration

      --dry-run
          print out the command invocation -- don't actually run it
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
