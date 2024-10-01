use std::{fs::File, io::Read};

use lang::{parse_program, ParseResult, Run};
use lex::Lexer;

mod lex;
mod lang;

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

    let parse_result = parse_program(run, &mut Vec::new());

    match parse_result {
        ParseResult::Err(_) => println!("error"),
        ParseResult::Ok(stmt, run) => println!("\nstmt: {:?}\ndist: {}\nops: {:?}", stmt, run.dist, run.ops),
    }
}
