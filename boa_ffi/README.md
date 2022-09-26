# boa_ffi -- Foreign Function Interface for Boa

This is meant to address boa-dev/boa#332

## Prioritized Work Items

- [x] Make C-compatible library
- [x] Generate C-compatible header via `cbindgen`
- [x] "Hello, World" -- call any function
- [ ] `exec` -- execute arbitrary JavaScript sent as `const char*` and returning `const char*`
- [ ] Automate cbindgen from build
- [ ] Integration tests to make sure it works end-to-end

## Open Questions

- [ ] C++ as well as C bindings?
- [ ] Where to put `cbindgen` output?
- [ ] What `boa_engine` features do we need? I have `boa_engine = { workspace = true, features = ["deser", "console"] }`

## Interesting Things

https://doc.rust-lang.org/nomicon/ffi.html

https://michael-f-bryan.github.io/rust-ffi-guide/cbindgen.html

cbindgen . --lang c --output target/boa.h
