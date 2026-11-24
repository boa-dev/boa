//! Native stack guard.
//!
//! The parser is recursive descent, so deeply nested input (`(((...`, `[[[...`, `{{{...`) overflows the
//! native stack of the parsing thread, which aborts the whole process. [`stack_low`] lets the parser fail
//! with an error instead once the stack is nearly used up.

/// Native stack that must be left for [`stack_low`] to report `false`, capped at a quarter of the thread's
/// stack so small stacks (musl's default thread stack, embedder worker threads) can still parse.
// Measured on x86_64 Linux and aarch64 macOS, the parser uses at most ~32 KiB of stack between two checks in
// debug builds and ~16 KiB in release builds.
const STACK_RED_ZONE: usize = 128 * 1024;

/// Returns `true` once less than the red zone of this thread's stack is left. Always `false` where the stack
/// bounds are unknown.
pub(crate) fn stack_low() -> bool {
    thread_local!(static LIMIT: usize = stack_bounds()
        .map_or(0, |(low, high)| low + STACK_RED_ZONE.min((high - low) / 4)));
    let here = 0u8;
    LIMIT.with(|limit| (&raw const here as usize) < *limit)
}

/// The current thread's stack as `(low, high)` addresses.
#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
fn stack_bounds() -> Option<(usize, usize)> {
    // SAFETY: only queries the attributes of the current thread; `attr` is destroyed after a
    // successful `pthread_getattr_np`.
    let (addr, size) = unsafe {
        let mut attr = std::mem::MaybeUninit::<libc::pthread_attr_t>::uninit();
        if libc::pthread_getattr_np(libc::pthread_self(), attr.as_mut_ptr()) != 0 {
            return None;
        }
        let (mut addr, mut size) = (std::ptr::null_mut(), 0);
        let rc = libc::pthread_attr_getstack(attr.as_ptr(), &raw mut addr, &raw mut size);
        libc::pthread_attr_destroy(attr.as_mut_ptr());
        if rc != 0 {
            return None;
        }
        (addr as usize, size)
    };
    let high = addr + size;

    // musl reports only the part of the main thread's stack that is mapped so far; it grows on demand up to
    // RLIMIT_STACK. glibc already accounts for the limit.
    #[cfg(target_env = "musl")]
    // SAFETY: plain syscalls; `rlimit` is written by `getrlimit` before it is read.
    let size = unsafe {
        if libc::gettid() == libc::getpid() {
            let mut rlimit = std::mem::MaybeUninit::<libc::rlimit>::uninit();
            if libc::getrlimit(libc::RLIMIT_STACK, rlimit.as_mut_ptr()) != 0 {
                return None;
            }
            let limit = rlimit.assume_init().rlim_cur;
            if limit == libc::RLIM_INFINITY {
                return None;
            }
            size.max(usize::try_from(limit).ok()?)
        } else {
            size
        }
    };

    Some((high.checked_sub(size)?, high))
}

/// The current thread's stack as `(low, high)` addresses.
#[cfg(target_vendor = "apple")]
#[allow(unsafe_code)]
fn stack_bounds() -> Option<(usize, usize)> {
    // SAFETY: only queries the current thread; `pthread_get_stackaddr_np` returns the stack's high end.
    let (high, size) = unsafe {
        let thread = libc::pthread_self();
        (
            libc::pthread_get_stackaddr_np(thread) as usize,
            libc::pthread_get_stacksize_np(thread),
        )
    };
    Some((high.checked_sub(size)?, high))
}

/// The current thread's stack as `(low, high)` addresses.
#[cfg(all(windows, not(target_vendor = "win7")))]
#[allow(unsafe_code, clippy::unnecessary_wraps)]
fn stack_bounds() -> Option<(usize, usize)> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentThreadStackLimits(low_limit: *mut usize, high_limit: *mut usize);
    }
    let (mut low, mut high) = (0, 0);
    // SAFETY: writes the current thread's stack bounds to the two locals.
    unsafe { GetCurrentThreadStackLimits(&raw mut low, &raw mut high) };
    Some((low, high))
}

/// The current thread's stack bounds are unknown on this platform.
#[cfg(not(any(
    target_os = "linux",
    target_vendor = "apple",
    all(windows, not(target_vendor = "win7"))
)))]
fn stack_bounds() -> Option<(usize, usize)> {
    None
}
