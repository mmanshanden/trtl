pub mod bencode;
pub mod highlight;


pub use bencode::benchode_highlight;
pub use highlight::highlight;

use crate::{lang::{ast::Program, compile::compile, lex::Lexer, parse::parse_program, run::Run}, machine::{canvas::{self, Canvas}, cpu::Cpu}};

/// Converts a wasm memory to a string
/// 
unsafe fn read_string_from_mem(ptr: *mut u8, len: usize) -> String {
    let bytes = Vec::from_raw_parts(ptr, len, len);
    String::from_utf8_lossy(&bytes).to_string()
}

/// Allows the provided bytes to be read by the client. 
/// 
/// Returns a pointer to the 8 byte array consisting of the 4 byte pointer that
/// points to the provided array of bytes and a 4 byte integer that marks the 
/// length of the provided bytes.
/// 
fn return_bytes(bytes: Vec<u8>) -> *const u8 {
    let ptr = bytes.as_ptr() as usize;
    let buf = vec![
        ptr,
        bytes.len(),
        bytes.capacity()
    ];

    let ptr = buf.as_ptr();

    std::mem::forget(buf);
    std::mem::forget(bytes);

    ptr as *const u8
}

extern "C" {
    fn print(start: usize, len: usize, cap: usize);
    fn alert(start: usize, len: usize, cap: usize);
}

fn console_log(msg: String) {
    let ptr = msg.as_ptr();
    let len = msg.len();
    let cap = msg.capacity();

    std::mem::forget(msg);

    unsafe {
        print(ptr as usize, len, cap);
    }
}

fn console_error(msg: String) {
    let ptr = msg.as_ptr();
    let len = msg.len();
    let cap = msg.capacity();

    std::mem::forget(msg);

    unsafe {
        alert(ptr as usize, len, cap);
    }
}

#[no_mangle]
pub unsafe fn malloc(len: usize) -> *const u8 {
    let buffer = Vec::with_capacity(len);
    let ptr = buffer.as_ptr();

    std::mem::forget(buffer);

    ptr
}

#[no_mangle]
pub unsafe fn mfree(ptr: *mut u8, len: usize) {
    let buffer = Vec::from_raw_parts(ptr, len, len);
    drop(buffer)
}

#[no_mangle]
pub unsafe fn syntax_fragments(ptr: *mut u8, len: usize) -> *const u8 {
    std::panic::set_hook(Box::new(|panic_info| {
        let out = panic_info.to_string();
        console_error(out)
    }));

    let input = read_string_from_mem(ptr, len);
    let fragments = highlight(&input);
    let mut bytes = Vec::new();
    
    benchode_highlight(&mut bytes, fragments);
    return_bytes(bytes)
}

#[no_mangle]
pub unsafe fn create_canvas(width: u32, height: u32) -> *mut Canvas {
    std::panic::set_hook(Box::new(|panic_info| {
        let out = panic_info.to_string();
        console_error(out)
    }));

    let canvas = Canvas::new(width, height);
    let canvas = Box::new(canvas);

    Box::into_raw(canvas)
}

#[no_mangle]
pub unsafe fn destroy_canvas(canvas: *mut Canvas) {
    drop(Box::from_raw(canvas))
}

#[no_mangle]
pub unsafe fn canvas_pixels(canvas: *mut Canvas) -> *const u8{
    let canvas = Box::from_raw(canvas);
    let pixels = canvas.get_pixel_data().clone();

    std::mem::forget(canvas);

    return_bytes(pixels)
}

#[no_mangle]
pub unsafe fn canvas_clear(canvas: *mut Canvas) {
    let mut canvas = Box::from_raw(canvas);
    canvas.clear();

    std::mem::forget(canvas);
}

#[no_mangle]
fn create_cpu(ptr: *mut u8, len: usize) -> *mut Cpu {
    std::panic::set_hook(Box::new(|panic_info| {
        let out = panic_info.to_string();
        console_error(out)
    }));

    let input = unsafe { 
        read_string_from_mem(ptr, len) 
    };

    let tokens = Lexer::new(&input).tokens();

    let ast = match parse_program(Run::new(&tokens)) {
        crate::lang::run::ParseResult::Err(_) => Program::new(),
        crate::lang::run::ParseResult::Ok(ast, _) => ast
    };

    let code = compile(ast).unwrap_or_default();

    let cpu = Cpu::new(|str| println!("{}", str), code);
    // let cpu = Cpu::new(|str| console_log(str.to_string()), code);
    let cpu = Box::new(cpu);

    Box::into_raw(cpu)
}

#[no_mangle]
pub unsafe fn destroy_cpu(cpu: *mut Cpu) {
    drop(Box::from_raw(cpu));
}

#[no_mangle]
pub unsafe fn cpu_run(cpu: *mut Cpu, canvas: *mut Canvas, n: u32) {
    std::panic::set_hook(Box::new(|panic_info| {
        let out = panic_info.to_string();
        console_error(out)
    }));

    let mut cpu = Box::from_raw(cpu);
    let mut canvas = Box::from_raw(canvas);

    cpu.run_n(&mut canvas, n);

    std::mem::forget(cpu);
    std::mem::forget(canvas);
}

#[no_mangle]
pub unsafe fn cpu_is_halted(cpu: *mut Cpu) -> u8 {
    let cpu = Box::from_raw(cpu);
    let halted = cpu.is_halted();

    std::mem::forget(cpu);

    halted as u8
}
