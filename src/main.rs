use core::str;
use std::{fs::File, io::Read};

use lang::{parse_program, Lexer, Run};


// use minifb::{Window, WindowOptions};

pub mod lang;
pub mod machine;

fn stdout(s: &str) {
    println!("{}", s);
}


fn main() {
    let source = "src/main.trs";

    let code = match File::open(source) {
        Result::Ok(mut file) => {
            let mut content = String::new();
            if let Result::Err(error) = file.read_to_string(&mut content) {
                println!("Error reading file {}: {}", source, error);
                return;
            }

            content
        }
        Result::Err(error) => {
            println!("Error reading file {}: {}", source, error);
            return;
        }
    };


    let mut lexer = Lexer::new(&code);
    let tokens = lexer.tokens();
    let run = Run::new(&tokens);

    let r = parse_program(run);

    println!("Parse result: {:?}", r);

    // let parse_result = parse_program(run);

    // let (program, dist, ops) = match parse_result {
    //     ParseResult::Err(_) => { 
    //         println!("error");
    //         return
    //     },
    //     ParseResult::Ok(program, run) => (program, run.dist, run.corrections),
    // };

    // let ops = compile(program);
    
    // =======================

    // let window_options = WindowOptions::default();

    // let mut canvas = Canvas::new(786, 786);
    // let mut cpu = Cpu::new(stdout, ops);
    // let mut window = Window::new("trtl", 786, 786, window_options).unwrap();

    // window.limit_update_rate(None);

    // while window.is_open() {
    //     cpu.run_n(&mut canvas, 1 << 18);

    //     window.update_with_buffer(&canvas.get_pixel_data(), 786, 786).unwrap();
    // }
}
