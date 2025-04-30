mod lex;
mod run;
mod parse;
mod ast;
mod compile;

pub use ast::*;
pub use lex::{Lexer, Token};
pub use run::{Run, Tag, Tags};
pub use compile::{compile, CompileError};

use parse::ParseResult;
use parse::{Deny, Parser};

pub fn parse_program<'a>(run: Run<'a>) -> Result<(Program, Tags<'a>), ()> {
    let deny = Deny::new();

    let result = match parse::parse_program().parse(run, deny) {
        ParseResult::Success(program, tags, _) => Ok((program, tags)),
        ParseResult::Error => Err(()),
    };

    result
}