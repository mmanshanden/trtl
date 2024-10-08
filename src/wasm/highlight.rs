use core::fmt;
use std::{collections::HashMap, hash::Hash, io::Cursor};

use crate::lang::{lex::{Lexer, Loc, Span, Token}, parse::parse_program, run::{Op, Run, Tokens}};

use super::console_log;

#[derive(Debug, Clone)]
pub struct Fragment {
    pub value: String,
    pub kind: u8   
}

pub type Line = Vec<Fragment>;

pub type Highlight = Vec<Line>; 

fn op_to_lint(op: Op) -> Option<Span<String>> {
    match op {
        Op::Insert(_, _) => None,
        Op::Remove(from, to) => Some(Span { 
            value: "lint".to_string(),
            from,
            to
        })
    }
}

struct TokenMap {
    map: HashMap<usize, u8>
}

impl TokenMap {
    pub fn new<'a>(tokens: Vec<Span<Token<'a>>>) -> Self {
        let mut map = HashMap::new();

        for token in tokens {
            let from = token.from.char;
            let to = token.to.char;
            let kind = match token.value {
                Token::If => 1,
                Token::Else => 1,
                Token::Func => 1,
                Token::While => 1,
                Token::Break => 2,
                Token::Return => 2,
                Token::True => 3,
                Token::False => 3,
                Token::Number(_) => 4,
                Token::Identifier(_) => 5,
                _ => 0
            };

            for i in from..to {
                map.insert(i, kind);
            }
        }

        TokenMap { 
            map 
        }
    }

    pub fn check(&self, idx: usize) -> Option<&u8> {
        self.map.get(&idx)
    }
}

pub fn highlight(input: &str) -> Highlight {
    let tokens = Lexer::new(input).tokens();
    let run = Run::new(&tokens);

    let ops = match parse_program(run) {
        crate::lang::run::ParseResult::Err(r) => r.ops,
        crate::lang::run::ParseResult::Ok(_, r) => r.ops
    };

    console_log(format!("{:?}", ops));

    let token_map = TokenMap::new(tokens);

    let mut current = Fragment {
        kind: *token_map.check(0).unwrap_or(&0),
        value: String::new()
    };

    let mut fragments = Vec::new();
    let mut highlights = Vec::new();

    for (i, char) in input.chars().enumerate() {
        let &kind = token_map.check(i).unwrap_or(&current.kind);

        if char ==  '\n' {
            fragments.push(current);
            highlights.push(fragments);
            
            fragments = Vec::new();
            current = Fragment {
                kind,
                value: String::new()
            };

            continue;
        }

        if kind == current.kind {
            current.value.push(char);
        } else {
            fragments.push(current);
            current = Fragment {
                kind,
                value: char.to_string()
            }
        }
    }

    fragments.push(current);
    highlights.push(fragments);

    highlights
}