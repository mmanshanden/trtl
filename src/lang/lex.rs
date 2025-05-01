/// The `Token` type
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Token<'a> {
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
    LineBreak
}


impl<'a> Token<'a> {
    pub fn identifier(&self) -> Option<&'a str> {
        match self {
            Self::Identifier(id) => Some(id),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<&'a str> {
        match self {
            Self::Number(num) => Some(num),
            _ => None,
        }
    }

    pub fn dist(&self) -> usize {
        match self {
            Self::Whitespace(_) => 0,
            Self::Comment(_) => 0,
            Self::LineBreak => 0,
            _ => 1,
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


#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    current_char: Option<&'a u8>,
    pos: Loc
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
            (0x09, _, _) | 
            (0x0B, _, _) |
            (0x0C, _, _) |
            (0x0D, _, _) |
            (0x20, _, _) | 
            (0xC2, 0x85, _) |
            (0xC2, 0xA0, _) | 
            (0xE1, 0x9A, 0x80) | 
            (0xE1, 0xA0, 0x8E) |
            (0xE2, 0x80, 0x80) |
            (0xE2, 0x80, 0x81) |
            (0xE2, 0x80, 0x82) |
            (0xE2, 0x80, 0x83) |
            (0xE2, 0x80, 0x84) |
            (0xE2, 0x80, 0x85) |
            (0xE2, 0x80, 0x86) |
            (0xE2, 0x80, 0x87) |
            (0xE2, 0x80, 0x88) |
            (0xE2, 0x80, 0x89) |
            (0xE2, 0x80, 0x8A) |
            (0xE2, 0x80, 0xA8) |
            (0xE2, 0x80, 0xA9) |
            (0xE2, 0x80, 0xAF) |
            (0xE2, 0x81, 0x9F) |
            (0xE3, 0x80, 0x80)
        )
    }

    fn advance(&mut self) {
        if let Some(&c) = self.current_char {
            self.pos.char += 1;

            if c > 0b11110000 {
                self.pos.byte += 4;
            } else if c > 0b11100000 {
                self.pos.byte += 3;
            } else if c > 0b11000000 {
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
        while self.current_char.map(|c| c.is_ascii_digit()).unwrap_or(false) {
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
            return Token::Eof;
        }

        let token = match self.current_char.unwrap() {
            b';' => {
                self.advance();
                Token::SemiColon
            }
            b'+' => {
                self.advance();
                Token::Plus
            }
            b'-' => {
                self.advance();
                Token::Minus
            }
            b'*' => {
                self.advance();
                Token::Multiply
            }
            b'/' => {
                self.advance();
                Token::Divide
            }
            b'=' => {
                self.advance();
                if self.current_char == Some(&b'=') {
                    self.advance();
                    Token::Equals
                } else {
                    Token::Assign
                }
            }
            b'!' => {
                self.advance();
                if self.current_char == Some(&b'=') {
                    self.advance();
                    Token::NotEqual
                } else {
                    Token::Bang
                }
            }
            b'>' => {
                self.advance();
                if self.current_char == Some(&b'=') {
                    self.advance();
                    Token::GreaterEqualThan
                } else {
                    Token::GreaterThan
                }
            }
            b'<' => {
                self.advance();
                if self.current_char == Some(&b'=') {
                    self.advance();
                    Token::LessEqualThan
                } else {
                    Token::LessThan
                }
            }
            b'(' => {
                self.advance();
                Token::LeftParen
            }
            b')' => {
                self.advance();
                Token::RightParen
            }
            b'{' => {
                self.advance();
                Token::LeftBrace
            }
            b'}' => {
                self.advance();
                Token::RightBrace
            }
            b',' => {
                self.advance();
                Token::Comma
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => match self.read_identifier() {
                "func" => Token::Func,
                "if" => Token::If,
                "else" => Token::Else,
                "while" => Token::While,
                "return" => Token::Return,
                "break" => Token::Break,
                "forward" => Token::Forward,
                "left" => Token::Left,
                "right" => Token::Right,
                "true" => Token::True,
                "false" => Token::False,
                id => Token::Identifier(id),
            },
            b'0'..=b'9' => {
                let num = self.read_number();
                Token::Number(num)
            }
            b'\n' => {
                self.advance();
                Token::LineBreak
            }
            _ if self.is_whitespace() => {
                let whitespace = self.read_whitespace();
                Token::Whitespace(whitespace)
            }
            _ => {
                let any = self.read_any();
                Token::Undefined(any)
            }
        };

        token
    }

    pub fn tokens(&mut self) -> Vec<Token<'a>> {
        let mut vec = Vec::new();
        let mut token = self.next_token();

        while token != Token::Eof {
            vec.push(token);
            token = self.next_token();
        }

        vec.push(token);
        vec
    }
}
