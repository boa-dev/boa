use boa_engine::Context;
use std::ffi::{c_char, CStr, CString};

#[no_mangle]
pub extern "C" fn boa_exec(src_bytes: *const c_char) -> *mut c_char {
    let c_str: &CStr = unsafe { CStr::from_ptr(src_bytes) };

    let return_value = match Context::default().eval(c_str.to_bytes()) {
        Ok(value) => value.display().to_string(),
        Err(error) => error.display().to_string(),
    };

    let s = CString::new(return_value).unwrap();
    s.into_raw()
}

#[no_mangle]
pub extern "C" fn boa_free_string(src_bytes: *mut c_char) {
    unsafe {
        if src_bytes.is_null() {
            return;
        }
        CString::from_raw(src_bytes)
    };
}
