use std::mem::offset_of;

/// The `Symbol` type
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Symbol<'a> {
    // util
    Undefined(&'a str),
    Eof,

    // variable
    Identifier(&'a str),
    Number(&'a str),

    // const
    True,
    False,

    // operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Assign,
    Bang,
    Equals,
    NotEqual,
    GreaterThan,
    GreaterEqualThan,
    LessThan,
    LessEqualThan,

    // program structure
    LeftParen,
    RightParen,
    Comma,
    SemiColon,
    LeftBrace,
    RightBrace,

    // reserved words
    If,
    Else,
    While,
    Return,
    Break,
    Forward,
    Left,
    Right,
    Func,

    Whitespace(&'a str),
    Comment(&'a str),
    LineBreak,
}

impl<'a> Symbol<'a> {
    pub fn identifier(&self) -> Option<&'a str> {
        match self {
            Symbol::Identifier(id) => Some(id),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<&'a str> {
        match self {
            Symbol::Number(num) => Some(num),
            _ => None,
        }
    }

    pub fn dist(&self) -> usize {
        match self {
            Symbol::Whitespace(_) => 0,
            Symbol::Comment(_) => 0,
            Symbol::LineBreak => 0,
            _ => 1,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Symbol::Undefined(s) => s.len(),
            Symbol::Identifier(s) => s.len(),
            Symbol::Number(s) => s.len(),
            Symbol::Whitespace(s) => s.len(),
            Symbol::Comment(s) => s.len(),
            Symbol::Assign => 1,
            Symbol::Bang => 1,
            Symbol::Plus => 1,
            Symbol::Minus => 1,
            Symbol::Multiply => 1,
            Symbol::Divide => 1,
            Symbol::SemiColon => 1,
            Symbol::LeftParen => 1,
            Symbol::RightParen => 1,
            Symbol::Comma => 1,
            Symbol::LeftBrace => 1,
            Symbol::RightBrace => 1,
            Symbol::Equals => 2,
            Symbol::NotEqual => 2,
            Symbol::GreaterThan => 1,
            Symbol::GreaterEqualThan => 2,
            Symbol::LessThan => 1,
            Symbol::LessEqualThan => 2,
            Symbol::If => 2,
            Symbol::Else => 4,
            Symbol::While => 5,
            Symbol::Return => 6,
            Symbol::Break => 5,
            Symbol::Forward => 7,
            Symbol::Left => 4,
            Symbol::Right => 5,
            Symbol::Func => 4,
            Symbol::True => 4,
            Symbol::False => 5,
            Symbol::LineBreak => 1,
            Symbol::Eof => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Loc {
    pub line: usize,
    pub col: usize,
    pub byte: usize,
    pub char: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    pub symbol: Symbol<'a>,
    pub index_from: usize,
    pub index_to: usize,
}

impl<'a> Token<'a> {
    pub fn identifier(&self) -> Option<&'a str> {
        match self.symbol {
            Symbol::Identifier(id) => Some(id),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<&'a str> {
        match self.symbol {
            Symbol::Number(num) => Some(num),
            _ => None,
        }
    }

    pub fn dist(&self) -> usize {
        self.symbol.dist()
    }

    pub fn is_whitespace(&self) -> bool {
        match self.symbol {
            Symbol::Whitespace(_) => true,
            Symbol::Comment(_) => true,
            Symbol::LineBreak => true,
            Symbol::Eof => true,
            _ => false,
        }
    }
}

pub type Tokens<'a> = &'a [Token<'a>];

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    current_char: Option<&'a u8>,
    pos: Loc,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            current_char: input.as_bytes().first(),
            pos: Loc {
                line: 0,
                col: 0,
                byte: 0,
                char: 0,
            },
        }
    }

    fn is_newline(&self) -> bool {
        self.current_char == Some(&b'\n')
    }

    fn is_whitespace(&self) -> bool {
        let index = self.pos.byte;

        let p = self.input.as_bytes().get(index).copied().unwrap_or(0);
        let q = self.input.as_bytes().get(index + 1).copied().unwrap_or(0);
        let r = self.input.as_bytes().get(index + 2).copied().unwrap_or(0);

        matches!(
            (p, q, r),
            (0x09, _, _)
                | (0x0B, _, _)
                | (0x0C, _, _)
                | (0x0D, _, _)
                | (0x20, _, _)
                | (0xC2, 0x85, _)
                | (0xC2, 0xA0, _)
                | (0xE1, 0x9A, 0x80)
                | (0xE1, 0xA0, 0x8E)
                | (0xE2, 0x80, 0x80)
                | (0xE2, 0x80, 0x81)
                | (0xE2, 0x80, 0x82)
                | (0xE2, 0x80, 0x83)
                | (0xE2, 0x80, 0x84)
                | (0xE2, 0x80, 0x85)
                | (0xE2, 0x80, 0x86)
                | (0xE2, 0x80, 0x87)
                | (0xE2, 0x80, 0x88)
                | (0xE2, 0x80, 0x89)
                | (0xE2, 0x80, 0x8A)
                | (0xE2, 0x80, 0xA8)
                | (0xE2, 0x80, 0xA9)
                | (0xE2, 0x80, 0xAF)
                | (0xE2, 0x81, 0x9F)
                | (0xE3, 0x80, 0x80)
        )
    }

    fn advance(&mut self) {
        if let Some(&c) = self.current_char {
            self.pos.char += 1;

            if c >= 0b11110000 {
                self.pos.byte += 4;
            } else if c >= 0b11100000 {
                self.pos.byte += 3;
            } else if c >= 0b11000000 {
                self.pos.byte += 2;
            } else {
                self.pos.byte += 1;
            }

            if c == b'\n' {
                self.pos.line += 1;
                self.pos.col = 0;
            } else {
                self.pos.col += 1;
            }
        }

        self.current_char = self.input.as_bytes().get(self.pos.byte);
    }

    fn read_whitespace(&mut self) -> &'a str {
        let start = self.pos.byte;

        while self.is_whitespace() {
            self.advance();
        }

        &self.input[start..self.pos.byte]
    }

    /// Returns true when given `char` cannot be part of an identifier
    /// string.
    ///
    fn is_forbidden_identifier_char(&self) -> bool {
        matches!(
            self.current_char.unwrap_or(&0),
            b',' | b'.' | b';' | b'(' | b')' | b'-' | b'+' | b'/' | b'*' | b'^' | b'='
        )
    }

    fn read_any(&mut self) -> &'a str {
        let start = self.pos.byte;

        if self.current_char.is_some() {
            self.advance();
        }

        &self.input[start..self.pos.byte]
    }

    fn read_comment(&mut self) -> &'a str {
        let start = self.pos.byte;

        loop {
            if self.current_char.is_none() {
                break;
            }

            if self.is_newline() {
                break;
            }

            self.advance();
        }

        &self.input[start..self.pos.byte]
    }

    fn read_identifier(&mut self) -> &'a str {
        let start = self.pos.byte;

        loop {
            if self.current_char.is_none() {
                break;
            }

            if self.is_whitespace() || self.is_newline() {
                break;
            }

            if self.is_forbidden_identifier_char() {
                break;
            }

            self.advance();
        }

        &self.input[start..self.pos.byte]
    }

    fn read_number(&mut self) -> &'a str {
        let start = self.pos.byte;

        // Everything before the delimiter
        while self
            .current_char
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            self.advance()
        }

        // The delimiter
        if self.current_char.map(|&c| c == b'.').unwrap_or(false) {
            self.advance()
        }

        // Everything after the delimiter
        while self.current_char.map(u8::is_ascii_digit).unwrap_or(false) {
            self.advance()
        }

        &self.input[start..self.pos.byte]
    }

    pub fn next_token(&mut self) -> Token<'a> {
        if self.current_char.is_none() {
            return Token {
                symbol: Symbol::Eof,
                index_from: self.pos.byte,
                index_to: 0,
            };
        }

        let offset = self.pos.byte;
        let token = match self.current_char.unwrap() {
            b';' => {
                self.advance();
                Symbol::SemiColon
            }
            b'+' => {
                self.advance();
                Symbol::Plus
            }
            b'-' => {
                self.advance();
                Symbol::Minus
            }
            b'*' => {
                self.advance();
                Symbol::Multiply
            }
            b'/' => {
                self.advance();

                if let Some(&b'/') = self.current_char {
                    self.advance();
                    let comment = self.read_comment();
                    Symbol::Comment(comment)
                } else {
                    Symbol::Divide
                }
            }
            b'=' => {
                self.advance();

                if self.current_char == Some(&b'=') {
                    self.advance();
                    Symbol::Equals
                } else {
                    Symbol::Assign
                }
            }
            b'!' => {
                self.advance();

                if self.current_char == Some(&b'=') {
                    self.advance();
                    Symbol::NotEqual
                } else {
                    Symbol::Bang
                }
            }
            b'>' => {
                self.advance();

                if self.current_char == Some(&b'=') {
                    self.advance();
                    Symbol::GreaterEqualThan
                } else {
                    Symbol::GreaterThan
                }
            }
            b'<' => {
                self.advance();

                if self.current_char == Some(&b'=') {
                    self.advance();
                    Symbol::LessEqualThan
                } else {
                    Symbol::LessThan
                }
            }
            b'(' => {
                self.advance();
                Symbol::LeftParen
            }
            b')' => {
                self.advance();
                Symbol::RightParen
            }
            b'{' => {
                self.advance();
                Symbol::LeftBrace
            }
            b'}' => {
                self.advance();
                Symbol::RightBrace
            }
            b',' => {
                self.advance();
                Symbol::Comma
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => match self.read_identifier() {
                "func" => Symbol::Func,
                "if" => Symbol::If,
                "else" => Symbol::Else,
                "while" => Symbol::While,
                "return" => Symbol::Return,
                "break" => Symbol::Break,
                "forward" => Symbol::Forward,
                "left" => Symbol::Left,
                "right" => Symbol::Right,
                "true" => Symbol::True,
                "false" => Symbol::False,
                id => Symbol::Identifier(id),
            },
            b'0'..=b'9' => {
                let num = self.read_number();
                Symbol::Number(num)
            }
            b'\n' => {
                self.advance();
                Symbol::LineBreak
            }
            _ if self.is_whitespace() => {
                let whitespace = self.read_whitespace();
                Symbol::Whitespace(whitespace)
            }
            _ => {
                let any = self.read_any();
                Symbol::Undefined(any)
            }
        };

        Token {
            symbol: token,
            index_from: offset,
            index_to: self.pos.byte,
        }
    }

    pub fn collect(&mut self) -> Vec<Token<'a>> {
        let mut vec = Vec::new();
        let mut token = self.next_token();

        while token.symbol != Symbol::Eof {
            vec.push(token);
            token = self.next_token();
        }

        vec.push(token);
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_and_number() {
        let mut lexer = Lexer::new("foo 123");
        let tokens = lexer.collect();
        // Expect: Identifier("foo"), Whitespace(" "), Number("123"), Eof
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].identifier(), Some("foo"));
        assert_eq!(tokens[2].number(), Some("123"));

        // indices: "foo" starts at 0..3, whitespace 3..4, number 4..7
        assert_eq!(tokens[0].index_from, 0);
        assert_eq!(tokens[0].index_to, 3);
        assert_eq!(tokens[1].index_from, 3);
        assert_eq!(tokens[1].index_to, 4);
        assert_eq!(tokens[2].index_from, 4);
        assert_eq!(tokens[2].index_to, 7);
    }

    #[test]
    fn test_whitespace_tokens() {
        let mut lexer = Lexer::new("a \n// comment\n42");
        let tokens = lexer.collect();

        assert_eq!(tokens[0].symbol, Symbol::Identifier("a"));
        assert_eq!(tokens[5].symbol, Symbol::Number("42"));

        assert!(tokens[1].is_whitespace()); // the space
        assert!(tokens[2].is_whitespace()); // the newline
        assert!(tokens[3].is_whitespace()); // the comment
        assert!(tokens[4].is_whitespace()); // the newline
        assert!(!tokens[0].is_whitespace()); // identifier "a"
        assert!(!tokens[5].is_whitespace()); // number "42"
    }

    #[test]
    fn test_operators_and_punctuators() {
        let input = "== != >= <= = ! > < ; + - * / ( ) { } ,";

        let mut lexer = Lexer::new(input);
        let tokens = lexer.collect();
        let symbols: Vec<Symbol> = tokens
            .iter()
            .filter(|token| !token.is_whitespace())
            .map(|t| t.symbol)
            .collect();

        let expected = vec![
            Symbol::Equals,
            Symbol::NotEqual,
            Symbol::GreaterEqualThan,
            Symbol::LessEqualThan,
            Symbol::Assign,
            Symbol::Bang,
            Symbol::GreaterThan,
            Symbol::LessThan,
            Symbol::SemiColon,
            Symbol::Plus,
            Symbol::Minus,
            Symbol::Multiply,
            Symbol::Divide,
            Symbol::LeftParen,
            Symbol::RightParen,
            Symbol::LeftBrace,
            Symbol::RightBrace,
            Symbol::Comma,
        ];

        assert_eq!(symbols.len(), expected.len());
        for (got, exp) in symbols.iter().zip(expected.iter()) {
            assert_eq!(got, exp);
        }
    }

    #[test]
    fn test_undefined_and_linebreak() {
        let mut lexer = Lexer::new("@\n");
        let tokens = lexer.collect();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].symbol, Symbol::Undefined("@"));
        assert_eq!(tokens[1].symbol, Symbol::LineBreak);
        assert_eq!(tokens[2].symbol, Symbol::Eof);
    }
}
