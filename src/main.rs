use core::str;
use std::{fs::File, io::Read};

use lang::{lex::Lexer, parse::parse_program, run::{ParseResult, Run}};

mod lang;
mod wasm;

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

    let parse_result = parse_program(run);

    let (program, dist, ops) = match parse_result {
        ParseResult::Err(_) => { 
            println!("error");
            return
        },
        ParseResult::Ok(program, run) => (program, run.dist, run.ops),
    };
}
