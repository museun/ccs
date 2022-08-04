# a simplier output for `cargo clippy`

```
Usage: ccs [OPTIONS]

Optional arguments:
  -h, --help
  -n, --nightly      use nightly (default: false)
  -l, --line-breaks  use line breaks (default: false)
  -w, --warn string  additional warning levels to use
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
