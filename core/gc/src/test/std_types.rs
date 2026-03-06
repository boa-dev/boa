use crate::{Gc, Trace, Tracer};
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use std::time::Instant;

#[test]
fn test_mutex_trace() {
    let mutex = Mutex::new(Gc::new(10));
    let mut tracer = Tracer::new();
    unsafe {
        mutex.trace(&mut tracer);
    }
    assert!(!tracer.is_empty(), "Mutex should trace its inner Gc value");
}

#[test]
fn test_rwlock_trace() {
    let rwlock = RwLock::new(Gc::new(20));
    let mut tracer = Tracer::new();
    unsafe {
        rwlock.trace(&mut tracer);
    }
    assert!(!tracer.is_empty(), "RwLock should trace its inner Gc value");
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
