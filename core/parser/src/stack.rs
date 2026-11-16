//! Native stack guard.
//!
//! The parser is recursive descent, so deeply nested input (`(((...`, `[[[...`, `{{{...`) overflows the
//! native stack of the parsing thread, which aborts the whole process. [`stack_low`] lets the parser fail
//! with an error instead once the stack is nearly used up.

/// Native stack that must be left for [`stack_low`] to report `false`.
#[cfg(target_os = "linux")]
const STACK_RED_ZONE: usize = 512 * 1024;

/// Returns `true` once less than `STACK_RED_ZONE` of this thread's stack is left.
#[cfg(target_os = "linux")]
pub(crate) fn stack_low() -> bool {
    thread_local!(static LIMIT: usize = stack_limit());
    let here = 0u8;
    LIMIT.with(|limit| (&raw const here as usize) < *limit)
}

/// Returns `true` once the stack is nearly used up (never: the stack bounds are unknown on this platform).
#[cfg(not(target_os = "linux"))]
pub(crate) const fn stack_low() -> bool {
    false
}

/// The lowest address the parser may reach: the stack's low end plus the red zone, or `0` if unknown.
#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
fn stack_limit() -> usize {
    // SAFETY: only queries the attributes of the current thread; `attr` is destroyed after a
    // successful `pthread_getattr_np`.
    unsafe {
        let mut attr = std::mem::MaybeUninit::<libc::pthread_attr_t>::uninit();
        if libc::pthread_getattr_np(libc::pthread_self(), attr.as_mut_ptr()) != 0 {
            return 0;
        }
        let (mut addr, mut size) = (std::ptr::null_mut(), 0);
        let rc = libc::pthread_attr_getstack(attr.as_ptr(), &raw mut addr, &raw mut size);
        libc::pthread_attr_destroy(attr.as_mut_ptr());
        if rc != 0 {
            return 0;
        }
        addr as usize + STACK_RED_ZONE
    }
}
