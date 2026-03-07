use crate::{Gc, Trace, Tracer};
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use std::time::Instant;

#[test]
fn test_sync_types_ignored_trace() {
    // Note: Since `Gc` pointers are `!Send`, they cannot be safely shared across threads.
    // Therefore, `Mutex` and `RwLock` just use `empty_trace!()` and ignore inner values.
    let mutex = Mutex::new(Gc::new(10));
    let rwlock = RwLock::new(Gc::new(20));

    let mut tracer = Tracer::new();

    unsafe {
        mutex.trace(&mut tracer);
        rwlock.trace(&mut tracer);
    }

    assert!(
        tracer.is_empty(),
        "Mutex and RwLock should not trace inner values because Gc is !Send"
    );
}

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
