use boa_engine::Context;
use core::ffi::{c_char, CStr};

#[no_mangle]
pub extern "C" fn boa_exec(src_bytes: *const c_char){
    let c_str: &CStr = unsafe { CStr::from_ptr(src_bytes) };

    let _return_value = match Context::default().eval(c_str.to_bytes()) {
        Ok(value) => value.display().to_string(),
        Err(error) => error.display().to_string(),
    }; // .as_bytes().as_mut_ptr();
}
