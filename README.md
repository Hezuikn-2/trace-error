```sh
sed -i 's/^    fn from(t: T) -> T {$/    default fn from(t: T) -> T {/' $(rustc --print sysroot)/lib/rustlib/src/rust/library/core/src/convert/mod.rs
```
