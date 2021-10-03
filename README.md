# bingen

Procedure macro for bringing a compile-time compiled assembly code as a binary slice.

```
let bin = bingen!("aarch64-linux-eabi", "mrs x0, DBGDTR_EL0");
assert_eq!(bin, [0, 4, 51, 213]);
```
