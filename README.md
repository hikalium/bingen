# bingen

Procedure macro for bringing a compile-time compiled assembly code as a binary slice.

```toml
[dependencies]
bingen = "0.2"
```

```
let bin = bingen!("aarch64-linux-eabi", "mrs x0, DBGDTR_EL0");
assert_eq!(bin, [0, 4, 51, 213]);
```

Currently, this crate supports macOS+Homebrew / Linux+llvm-8 environments by default.

If you want to use other llvm installation, specify the paths manually like this:
```
BINGEN_CLANG_PATH=/path/to/clang \
  BINGEN_OBJCOPY_PATH=/path/to/llvm-objcopy \
  cargo test
```
