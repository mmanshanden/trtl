use core::fmt;

use crate::lang::lex::{Lexer, Span, Token};

use super::console_log;

pub struct Fragment {
    pub from: usize,
    pub to: usize,
    pub kind: usize   
}

pub fn highlight(input: &str) -> Vec<Fragment> {
    let mut lexer = Lexer::new(input);

    let mut pos = 0;
    let mut fragments = Vec::new();

    loop {
        let token = lexer.next_token();

        if pos < token.from.byte {
            fragments.push(Fragment {
                from: pos,
                to: token.from.byte,
                kind: 0
            });
        }

        if token.value == Token::Eof {
            break;
        }

        let kind = match token.value {
            Token::If => 1,
            Token::Else => 1,
            Token::Func => 1,
            Token::While => 1,
            Token::Break => 2,
            Token::Return => 2,
            Token::True => 3,
            Token::False => 3,
            Token::Identifier(_) => 4,
            Token::Number(_) => 5,
            _ => 0
        };

        fragments.push(Fragment {
            from: token.from.byte,
            to: token.to.byte,
            kind
        });

        pos = token.to.byte
    }

    fragments
}