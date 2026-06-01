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

const HEADER_SIZE: usize = 16;
const ALIGN: usize = 16;

fn align_size(size: usize) -> usize {
    (size + 15) & !15
}

#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    let size = if size == 0 { 1 } else { size };
    let aligned = align_size(size);
    let layout = Layout::from_size_align_unchecked(aligned + HEADER_SIZE, ALIGN);
    let ptr = alloc(layout);
    if ptr.is_null() {
        return ptr as *mut c_void;
    }
    *(ptr as *mut usize) = aligned;
    ptr.add(HEADER_SIZE) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let ptr = (ptr as *mut u8).sub(HEADER_SIZE);
    let size = *(ptr as *mut usize);
    let layout = Layout::from_size_align_unchecked(size + HEADER_SIZE, ALIGN);
    dealloc(ptr, layout);
}

#[no_mangle]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
    let mut total = nmemb * size;
    if total == 0 {
        total = 1;
    }
    let aligned = align_size(total);
    let layout = Layout::from_size_align_unchecked(aligned + HEADER_SIZE, ALIGN);
    let ptr = alloc_zeroed(layout);
    if ptr.is_null() {
        return ptr as *mut c_void;
    }
    *(ptr as *mut usize) = aligned;
    ptr.add(HEADER_SIZE) as *mut c_void
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
    let orig_ptr = (ptr as *mut u8).sub(HEADER_SIZE);
    let old_size = *(orig_ptr as *mut usize);
    let layout = Layout::from_size_align_unchecked(old_size + HEADER_SIZE, ALIGN);

    let aligned = align_size(size);
    let new_ptr = rs_realloc(orig_ptr, layout, aligned + HEADER_SIZE);
    if new_ptr.is_null() {
        return new_ptr as *mut c_void;
    }
    *(new_ptr as *mut usize) = aligned;
    new_ptr.add(HEADER_SIZE) as *mut c_void
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
