use std::alloc::{alloc, alloc_zeroed, dealloc, realloc as rs_realloc, Layout};
use std::os::raw::{c_int, c_void};

extern "C" {
    fn _wasm_c_stubs_init();
}

pub fn init_stubs() {
    unsafe {
        _wasm_c_stubs_init();
    }
}

#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    if size == 0 {
        return std::ptr::null_mut();
    }
    let layout = Layout::from_size_align_unchecked(size + 8, 8);
    let ptr = alloc(layout);
    if ptr.is_null() {
        return ptr as *mut c_void;
    }
    *(ptr as *mut usize) = size;
    ptr.add(8) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let ptr = (ptr as *mut u8).sub(8);
    let size = *(ptr as *mut usize);
    let layout = Layout::from_size_align_unchecked(size + 8, 8);
    dealloc(ptr, layout);
}

#[no_mangle]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
    let total = nmemb * size;
    if total == 0 {
        return std::ptr::null_mut();
    }
    let layout = Layout::from_size_align_unchecked(total + 8, 8);
    let ptr = alloc_zeroed(layout);
    if ptr.is_null() {
        return ptr as *mut c_void;
    }
    *(ptr as *mut usize) = total;
    ptr.add(8) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    if ptr.is_null() {
        return malloc(size);
    }
    if size == 0 {
        free(ptr);
        return std::ptr::null_mut();
    }
    let orig_ptr = (ptr as *mut u8).sub(8);
    let old_size = *(orig_ptr as *mut usize);
    let layout = Layout::from_size_align_unchecked(old_size + 8, 8);
    let new_ptr = rs_realloc(orig_ptr, layout, size + 8);
    if new_ptr.is_null() {
        return new_ptr as *mut c_void;
    }
    *(new_ptr as *mut usize) = size;
    new_ptr.add(8) as *mut c_void
}

#[no_mangle]
pub extern "C" fn iswspace(c: c_int) -> c_int {
    let ch = match std::char::from_u32(c as u32) {
        Some(ch) => ch,
        None => return 0,
    };
    if ch.is_whitespace() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn iswalnum(c: c_int) -> c_int {
    let ch = match std::char::from_u32(c as u32) {
        Some(ch) => ch,
        None => return 0,
    };
    if ch.is_alphanumeric() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn towupper(c: c_int) -> c_int {
    let ch = match std::char::from_u32(c as u32) {
        Some(ch) => ch,
        None => return c,
    };
    ch.to_uppercase().next().unwrap() as c_int
}

#[no_mangle]
pub extern "C" fn __assert_fail(
    assertion: *const u8,
    file: *const u8,
    line: u32,
    function: *const u8,
) -> ! {
    unsafe {
        let assertion_str = std::ffi::CStr::from_ptr(assertion as *const i8).to_string_lossy();
        let file_str = std::ffi::CStr::from_ptr(file as *const i8).to_string_lossy();
        let function_str = std::ffi::CStr::from_ptr(function as *const i8).to_string_lossy();
        panic!(
            "Assertion failed: {} at {}:{} in {}",
            assertion_str, file_str, line, function_str
        );
    }
}

#[no_mangle]
pub extern "C" fn rs_abort() -> ! {
    panic!("C abort() called via rs_abort()");
}
