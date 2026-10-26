//! Regenerates the checked-in C headers for `boa_ffi`.

use std::{fs, io, path::Path};

use diplomat_tool::{DocsUrlGenerator, config::Config};

fn main() -> io::Result<()> {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ffi/boa_ffi");
    let headers = crate_dir.join("c/generated");

    // This directory contains only generated files. Remove obsolete headers too.
    match fs::remove_dir_all(&headers) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    let mut config = Config::default();
    config.shared_config.lib_name = Some("boa_ffi".into());
    diplomat_tool::r#gen(
        &crate_dir.join("src/lib.rs"),
        "c",
        &headers,
        &DocsUrlGenerator::default(),
        config,
        false,
    )
}
