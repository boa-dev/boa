use crate::{Trace, Tracer};
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn test_simple_types_trace() {
    let mut tracer = Tracer::new();
    unsafe {
        Instant::now().trace(&mut tracer);
        PathBuf::from(".").trace(&mut tracer);
    }
    assert!(
        tracer.is_empty(),
        "Simple types should not add anything to tracer"
    );
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn test_file_trace() {
    use std::fs::File;
    if let Ok(file) = File::open("Cargo.toml") {
        let mut tracer = Tracer::new();
        unsafe {
            file.trace(&mut tracer);
        }
        assert!(
            tracer.is_empty(),
            "File handle should not add anything to tracer"
        );
    }
}
