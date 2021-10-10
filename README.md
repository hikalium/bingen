# bingen

Procedure macro for bringing a compile-time compiled assembly code as a binary slice.

```toml
[dependencies]
bingen = "0.3"
```

```rust
let bin = bingen!("aarch64-linux-eabi", "mrs x0, DBGDTR_EL0");
assert_eq!(bin, [0, 4, 51, 213]);
```

This crate automatically detects the latest installation of llvm toolchains by default.

If you want to use other llvm installation, specify the paths manually like this:
```shell
BINGEN_CLANG_PATH=/path/to/clang \
  BINGEN_OBJCOPY_PATH=/path/to/llvm-objcopy \
  cargo test
```

To know which toolchain is used by default, run this:
```
$ cargo test --lib tests::print_llvm_path -- --nocapture --ignored
    Finished test [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests (target/debug/deps/bingen-4b08a2f2272400c9)

running 1 test
LLVMPath { clang: "/usr/bin/clang-8", llvm_objcopy: "/usr/bin/llvm-objcopy-8" }
test tests::print_llvm_path ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```
