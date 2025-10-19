mod ast;
mod compile;
mod grammar;
mod lex;
mod parse;
mod run;

pub use ast::*;
pub use compile::compile;
pub use lex::{Lexer, Symbol};
pub use run::{ReadResult, Run};

use parse::Deny;
use parse::ParseResult;

pub fn parse_program<'a>(run: Run<'a>) -> Result<(Program, Markers<'a>), ()> {
    let deny = Deny::new();

    let result = match parse::parse_program().parse(run, deny) {
        ParseResult::Success(program, tags, _) => Ok((program, tags)),
        ParseResult::Error => Err(()),
    };

    result
}
