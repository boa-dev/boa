#![deny(unsafe_op_in_unsafe_fn)]

use boa_engine::Context;
use std::ffi::{c_char, CStr, CString};

/// Evaluates the given code by compiling down to bytecode, interpreting the
/// bytecode into a value, then converting the value to a UTF-8 encoded C
/// string using the default context
///
/// The returned pointer should be freed with `boa_free_string`
///
/// # Arguments
///
/// * `src_bytes` - A zero-terminated UTF-8 encoded string
///
/// # Safety
///
/// The `src_bytes` argument better not be a bad pointer or bad things will happen.
#[no_mangle]
pub unsafe extern "C" fn boa_exec(src_bytes: *const c_char) -> *mut c_char {
    let c_str: &CStr = unsafe { CStr::from_ptr(src_bytes) };

    let return_value = match Context::default().eval(c_str.to_bytes()) {
        Ok(value) => value.display().to_string(),
        Err(error) => error.display().to_string(),
    };

    let s = CString::new(return_value).unwrap();
    s.into_raw()
}

/// Free the string result from a previous call to `boa_exec`
///
/// # Arguments
///
/// * `src_bytes` - The string pointer from a previous call to `boa_exec`
///
/// # Safety
///
/// The `src_bytes` argument better not be a bad pointer or bad things will happen.
#[no_mangle]
pub unsafe extern "C" fn boa_free_string(src_bytes: *mut c_char) {
    unsafe {
        if src_bytes.is_null() {
            return;
        }
        CString::from_raw(src_bytes)
    };
}

#[cfg(test)]
mod tests;
