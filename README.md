# a simplier output for `cargo clippy`

```
Usage: ccs [OPTIONS]

simplifies the output of cargo clippy

this runs clippy and produces are smaller output
with the `-e` flag, it'll also try to provide some context

Optional arguments:
  -h, --help            prints the help message
  -v, --version         prints the current version of this tool
  -e, --explain         use the `explain` format
  -t, --tests           check only test targets
  -a, --all-targets     check all targets
  -p, --path <path>     path to a specific Cargo.toml manifest. this defaults to the `cwd`
  -W, --warn <string>   additional warning lints to use
  -A, --allow <string>  additional allow lints to use
  -D, --deny <string>   additional deny lints to use
  -y, --annoying        use `clippy::all` and `clippy::nursery` (and nightly clippy) (default: false)  -n, --nightly         use nightly (default: false)
```

example:

> ccs

```
E0424 expected value, found module `self`
 ⮡ crates\shaken\src\modules\spotify\mod.rs:88:29
E0424 expected value, found module `self`
 ⮡ crates\shaken\src\modules\spotify\mod.rs:92:29
E0424 expected value, found module `self`
 ⮡ crates\shaken\src\modules\spotify\mod.rs:94:13
E0424 expected value, found module `self`
 ⮡ crates\shaken\src\modules\spotify\mod.rs:103:21
E0424 expected value, found module `self`
 ⮡ crates\shaken\src\modules\spotify\mod.rs:111:21
E0107 missing generics for struct `binding::Binding`
 ⮡ crates\shaken\src\modules\spotify\mod.rs:19:57
```

vs

> cargo clippy --message-format=short

```
crates\shaken\src\modules\spotify\mod.rs:88:29: error[E0424]: expected value, found module `self`
crates\shaken\src\modules\spotify\mod.rs:92:29: error[E0424]: expected value, found module `self`
crates\shaken\src\modules\spotify\mod.rs:94:13: error[E0424]: expected value, found module `self`
crates\shaken\src\modules\spotify\mod.rs:103:21: error[E0424]: expected value, found module `self`
crates\shaken\src\modules\spotify\mod.rs:111:21: error[E0424]: expected value, found module `self`
crates\shaken\src\modules\spotify\mod.rs:19:57: error[E0107]: missing generics for struct `binding::Binding`
error: could not compile `shaken` due to 6 previous errors
```
