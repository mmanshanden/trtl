pub mod bencode;
pub mod highlight;

use std::alloc::Layout;

use bencode::benchode_highlight;
use highlight::highlight;

use crate::{lang::{ast::Program, compile::compile, lex::Lexer, parse::parse_program, run::Run}, machine::{canvas::Canvas, cpu::Cpu}};

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
    let fragments = highlight(&input);
    benchode_highlight(&mut bytes, fragments);

    return_bytes(bytes)
}

#[no_mangle]
fn create_cpu(ptr: *mut u8, len: usize) -> *mut Cpu {
    console_log("creating cpu".to_string());
    std::panic::set_hook(Box::new(|panic_info| {
        let out = panic_info.to_string();
        console_error(&out)
    }));

    let input = unsafe { 
        read_string_from_mem(ptr, len) 
    };

    let tokens = Lexer::new(&input).tokens();

    let ast = match parse_program(Run::new(&tokens)) {
        crate::lang::run::ParseResult::Err(_) => Program::new(),
        crate::lang::run::ParseResult::Ok(ast, _) => ast
    };

    let code = compile(ast);

    let cpu = Cpu::new(|str| console_log(str.to_string()), code);
    let cpu = Box::new(cpu);

    Box::into_raw(cpu)
}

#[no_mangle]
pub unsafe fn destroy_cpu(cpu: *mut Cpu) {
    drop(Box::from_raw(cpu));
}

#[no_mangle]
pub unsafe fn cpu_exec(cpu: *mut Cpu, width: u32, height: u32, n: u32) -> *const u8 {
    let mut cpu = Box::from_raw(cpu);

    console_log(n.to_string());
    let mut canvas = Canvas::new(width, height);
    cpu.run_n(&mut canvas, n);

    std::mem::forget(cpu);

    let pixels = canvas.get_pixel_data();
    let bytes = pixels.into_iter().flat_map(|pixel| pixel.to_le_bytes()).collect();
    return_bytes(bytes) 
}
