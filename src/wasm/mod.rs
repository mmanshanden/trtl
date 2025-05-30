pub mod bencode;
pub mod highlight;

pub use bencode::benchode_highlight;
pub use highlight::highlight;

use crate::lang::{compile, parse_program, Lexer, Program, Run};
use crate::machine::{canvas::Canvas, cpu::Cpu};


/// Reads a section of wasm memory into a `String`.
/// 
fn read_string_from_mem(ptr: *mut u8, len: usize) -> String {
    let bytes = unsafe { Vec::from_raw_parts(ptr, len, len) };
    String::from_utf8_lossy(&bytes).to_string()
}

/// Returns a pointer to a byte array that has a fixed size of 12 bytes. 
/// The first 4 byte integer is the pointer to the provided bytes, the next
/// 4 byte integer is the length in bytes, and the last 4 byte integer is 
/// the capcity of the provided byte array, to be used by the client when
/// freeing the memory.
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

unsafe extern "C" {
    fn print(start: usize, len: usize, cap: usize);
    fn alert(start: usize, len: usize, cap: usize);
}

pub fn console_log(msg: String) {
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

#[unsafe(no_mangle)]
pub fn malloc(len: usize) -> *const u8 {
    let buffer = Vec::with_capacity(len);
    let ptr = buffer.as_ptr();

    std::mem::forget(buffer);

    ptr
}

#[unsafe(no_mangle)]
pub fn mfree(ptr: *mut u8, len: usize) {
    let buffer = unsafe { Vec::from_raw_parts(ptr, len, len) };
    drop(buffer)
}

#[unsafe(no_mangle)]
pub fn syntax_fragments(ptr: *mut u8, len: usize) -> *const u8 {
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

#[unsafe(no_mangle)]
pub fn create_canvas(width: u32, height: u32) -> *mut Canvas {
    std::panic::set_hook(Box::new(|panic_info| {
        let out = panic_info.to_string();
        console_error(out)
    }));

    let canvas = Canvas::new(width, height);
    let canvas = Box::new(canvas);

    Box::into_raw(canvas)
}

#[unsafe(no_mangle)]
pub fn destroy_canvas(canvas: *mut Canvas) {
    let canvas = unsafe { Box::from_raw(canvas) };
    drop(canvas)
}

#[unsafe(no_mangle)]
pub fn canvas_pixels(canvas: *mut Canvas) -> *const u8{
    let canvas = unsafe { Box::from_raw(canvas) };
    let pixels = canvas.get_pixel_data().clone();

    std::mem::forget(canvas);

    return_bytes(pixels)
}

#[unsafe(no_mangle)]
pub fn canvas_clear(canvas: *mut Canvas) {
    let mut canvas = unsafe { Box::from_raw(canvas) };
    canvas.clear();

    std::mem::forget(canvas);
}

#[unsafe(no_mangle)]
fn create_cpu(ptr: *mut u8, len: usize) -> *mut Cpu {
    std::panic::set_hook(Box::new(|panic_info| {
        let out = panic_info.to_string();
        console_error(out)
    }));

    let input = read_string_from_mem(ptr, len);
    let tokens = Lexer::new(&input).tokens();
    let run = Run::new(&tokens);

    let program = match parse_program(run) {
        Err(_) => Program::new(),
        Ok((program, _)) => program
    };

    let code = compile(program).unwrap_or_default();

    let cpu = Cpu::new(|str| console_log(str.to_string()), code);
    let cpu = Box::new(cpu);

    Box::into_raw(cpu)
}

#[unsafe(no_mangle)]
pub fn destroy_cpu(cpu: *mut Cpu) {
    let cpu = unsafe { Box::from_raw(cpu) };
    drop(cpu);
}

#[unsafe(no_mangle)]
pub fn cpu_run(cpu: *mut Cpu, canvas: *mut Canvas, n: u32) {
    std::panic::set_hook(Box::new(|panic_info| {
        let out = panic_info.to_string();
        console_error(out)
    }));

    let mut cpu = unsafe { Box::from_raw(cpu) };
    let mut canvas = unsafe { Box::from_raw(canvas) };

    cpu.run_n(&mut canvas, n);

    std::mem::forget(cpu);
    std::mem::forget(canvas);
}

#[unsafe(no_mangle)]
pub fn cpu_is_halted(cpu: *mut Cpu) -> u8 {
    let cpu = unsafe {
        Box::from_raw(cpu)
    };

    let halted = cpu.is_halted();

    std::mem::forget(cpu);

    halted as u8
}
