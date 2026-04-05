//! Build script for the `boa_engine` crate to detect if the toolchain is nightly.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Tell rustc that `cfg(boa_nightly)` is intentional
    println!("cargo:rustc-check-cfg=cfg(boa_nightly)");

    if rustversion::cfg!(since(2026 - 01 - 26)) {
        println!("cargo:rustc-cfg=boa_nightly");
    }
}
