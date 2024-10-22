use core::fmt;
use std::{collections::HashMap};

use crate::lang::{lex::{Lexer, Span, Token}, parse::parse_program, run::{Correction, Corrections, Run, Tokens}};


#[derive(Debug, Clone)]
pub struct Fragment {
    pub value: String,
    pub hint: u8,
    pub kind: u8,
}

pub type Line = Vec<Fragment>;

pub struct Highlight {
    pub lines: Vec<Line>,
    pub hints: Vec<String>
} 
    

struct TokenMap {
    chars: HashMap<usize, u8>
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
            chars: map 
        }
    }

    pub fn check(&self, idx: usize) -> Option<&u8> {
        self.chars.get(&idx)
    }
}

struct CorrectionMap {
    chars: HashMap<usize, u8>,
    hints: Vec<String>
}

impl CorrectionMap {
    pub fn new<'a>(corrections: Corrections<'a>) -> Self {
        let mut chars = HashMap::new();
        let mut hints = vec![String::new(), String::new()];

        corrections.map(|c| {
            // insert corrections into map: 
            // "location of character" -> "index of hint"
            // 
            // Hint index starts at 1 because 0 is reserved hard errors
            // that do not have a hint.
            match c {
                Correction::Insert(at, replacement) => {
                    chars.insert(at.char, hints.len() as u8);
                    hints.push(format!("Did you mean to put a {:?}?", replacement));
                },
                Correction::Remove(from, to) => {
                    for i in from.char..to.char {
                        chars.insert(i, 1);
                    }
                }
            }
        });

        CorrectionMap { 
            chars,
            hints
        }
    }

    pub fn check(&self, idx: usize) -> Option<&u8> {
        self.chars.get(&idx)
    }
}

pub fn highlight(input: &str) -> Highlight {
    let tokens = Lexer::new(input).tokens();
    let corrections = match parse_program(Run::new(&tokens)) {
        crate::lang::run::ParseResult::Err(r) => r.corrections,
        crate::lang::run::ParseResult::Ok(_, r) => r.corrections
    };

    let correction_map = CorrectionMap::new(corrections);
    let token_map = TokenMap::new(tokens);

    let mut current = Fragment {
        kind: *token_map.check(0).unwrap_or(&0),
        hint: *correction_map.check(0).unwrap_or(&0),
        value: String::new()
    };

    let mut fragments = Vec::new();
    let mut lines = Vec::new();

    for (i, char) in input.chars().enumerate() {
        let &hint = correction_map.check(i).unwrap_or(&0);
        let &kind = token_map.check(i).unwrap_or(&current.kind);

        if char == '\n' {
            // transition to new line
            fragments.push(current);
            lines.push(fragments);
            
            fragments = Vec::new();
            current = Fragment {
                kind,
                hint,
                value: String::new()
            };

            continue;
        }

        if kind != current.kind || hint != current.hint {
            if !current.value.is_empty() {
                fragments.push(current);
            }
                
            current = Fragment {
                kind,
                hint,
                value: char.to_string()
            };

            continue;
        }
       
        current.value.push(char);
    }

    fragments.push(current);
    lines.push(fragments);
    
    Highlight {
        lines,
        hints: correction_map.hints
    }
}