use core::fmt;
use std::{collections::HashMap, hash::Hash, io::Cursor};

use crate::lang::{lex::{Lexer, Loc, Span, Token}, parse::parse_program, run::{Correction, Corrections, Run, Tokens}};

use super::console_log;

#[derive(Debug, Clone)]
pub struct Fragment {
    pub value: String,
    pub lint: u8,
    pub kind: u8,
}

pub type Line = Vec<Fragment>;

pub type Highlight = Vec<Line>; 

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

struct CorrectionMap {
    map: HashMap<usize, u8>
}

impl CorrectionMap {
    pub fn new<'a>(corrections: Corrections<'a>) -> Self {
        let mut map = HashMap::new();

        corrections.map(|c| {
            match c {
                Correction::Insert(at, _) => {
                    map.insert(at.char, 1);
                },
                Correction::Remove(from, to) => {
                    for i in from.char..to.char {
                        map.insert(i, 2);
                    }
                }
            }
        });

        CorrectionMap { 
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
    
    let corrections = match parse_program(run) {
        crate::lang::run::ParseResult::Err(r) => r.corrections,
        crate::lang::run::ParseResult::Ok(_, r) => r.corrections
    };

    let correction_map = CorrectionMap::new(corrections);
    let token_map = TokenMap::new(tokens);

    let mut current = Fragment {
        kind: *token_map.check(0).unwrap_or(&0),
        lint: *correction_map.check(0).unwrap_or(&0),
        value: String::new()
    };

    let mut fragments = Vec::new();
    let mut highlights = Vec::new();

    for (i, char) in input.chars().enumerate() {
        
        let &lint = correction_map.check(i).unwrap_or(&0);
        let &kind = token_map.check(i).unwrap_or(&current.kind);

        if char == '\n' {
            fragments.push(current);
            highlights.push(fragments);
            
            fragments = Vec::new();
            current = Fragment {
                kind,
                lint,
                value: String::new()
            };

            continue;
        }

        if kind == current.kind && lint == current.lint {
            current.value.push(char);
        } else {
            fragments.push(current);
            current = Fragment {
                kind,
                lint,
                value: char.to_string()
            }
        }
    }

    fragments.push(current);
    highlights.push(fragments);

    highlights
}