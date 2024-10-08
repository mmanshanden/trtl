pub mod bencode;
pub mod highlight;

use std::alloc::Layout;

use bencode::benchode_highlight;
use highlight::highlight;

use crate::lang::{lex::Lexer, parse::parse_program};

/// Converts a wasm memory to a string
/// 
unsafe fn read_string_from_mem(ptr: *mut u8, len: usize) -> String {
    let bytes = Vec::from_raw_parts(ptr, len, len);
    std::str::from_utf8(&bytes).unwrap().to_string()
}

/// Allows the provided bytes to be read by the client. 
/// 
/// Returns a pointer to the 8 byte array consisting of the 4 byte pointer that
/// points to the provided array of bytes and a 4 byte integer that marks the 
/// length of the provided bytes.
/// 
fn return_bytes(bytes: Vec<u8>) -> *const u8 {
    let buf = vec![bytes.as_ptr() as usize, bytes.len()];
    let ptr = buf.as_ptr();

    std::mem::forget(buf);
    std::mem::forget(bytes);

    ptr as *const u8
}

extern "C" {
    fn print(start: usize, len: usize);
    fn alert(start: usize, len: usize);
}

fn console_log(msg: String) {
    let ptr = msg.as_ptr();
    let len = msg.len();
    unsafe {
        print(ptr as usize, len);
    }
}

fn console_error(msg: &str) {
    let ptr = msg.as_ptr();
    let len = msg.len();
    unsafe {
        alert(ptr as usize, len);
    }
}



#[no_mangle]
pub unsafe fn malloc(len: usize) -> *const u8 {
    let align = std::mem::align_of::<usize>();
    let layout = Layout::from_size_align_unchecked(len, align);

    std::alloc::alloc_zeroed(layout)
}

#[no_mangle]
pub unsafe fn mfree(ptr: *mut u8, len: usize) {
    let align = std::mem::align_of::<usize>();
    let layout = Layout::from_size_align_unchecked(len, align);
    std::alloc::dealloc(ptr, layout);
}

#[no_mangle]
fn high(ptr: *mut u8, len: usize) -> *const u8 {
    std::panic::set_hook(Box::new(|panic_info| {
        let out = panic_info.to_string();
        console_error(&out)
    }));

    let input = unsafe { 
        read_string_from_mem(ptr, len) 
    };

    let mut bytes = Vec::new();
    benchode_highlight(&mut bytes, highlight(&input));

    return_bytes(bytes)
}
