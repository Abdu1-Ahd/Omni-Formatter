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

const ALIGN: usize = 16;
const HEADER_SIZE: usize = 16;
const MAGIC: usize = 0x1BADB002;

#[inline]
fn align_size(size: usize) -> usize {
    (size + ALIGN - 1) & !(ALIGN - 1)
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
    *(ptr.add(4) as *mut usize) = MAGIC;
    ptr.add(HEADER_SIZE) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let orig_ptr = (ptr as *mut u8).sub(HEADER_SIZE);
    let magic = *(orig_ptr.add(4) as *mut usize);
    if magic != MAGIC {
        return;
    }
    let old_size = *(orig_ptr as *mut usize);
    let layout = Layout::from_size_align_unchecked(old_size + HEADER_SIZE, ALIGN);
    dealloc(orig_ptr, layout);
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
    *(ptr.add(4) as *mut usize) = MAGIC;
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
    let magic = *(orig_ptr.add(4) as *mut usize);

    let new_ptr = malloc(size);
    if new_ptr.is_null() {
        return std::ptr::null_mut();
    }

    if magic != MAGIC {
        let copy_size = std::cmp::min(size, 256);
        std::ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, copy_size);
        return new_ptr;
    }

    let old_user_size = *(orig_ptr as *mut usize);
    let copy_size = if size < old_user_size {
        size
    } else {
        old_user_size
    };
    std::ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, copy_size);

    free(ptr);

    new_ptr
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
