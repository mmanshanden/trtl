use std::{fmt::Debug, str::Chars};

/// The `Token` type
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Token<'a> {
    // util
    Undefined,
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
}

impl<'a> Token<'a> {
    pub fn identifier(&self) -> Option<&'a str> {
        match self {
            Self::Identifier(id) => Some(id),
            _ => None,
        }
    }

    // pub fn number(&self) -> Option<&'a str> {
    //     match self {
    //         Self::Number(num) => Some(num),
    //         _ => None,
    //     }
    // }

    // pub fn brace(&self) -> Option<Self> {
    //     match self {
    //         Self::LeftBrace => Some(Self::LeftBrace),
    //         Self::RightBrace => Some(Self::RightBrace),
    //         _ => None,
    //     }
    // }

    // pub fn parenthesis(&self) -> Option<Self> {
    //     match self {
    //         Self::LeftParen => Some(Self::LeftParen),
    //         Self::RightParen => Some(Self::RightParen),
    //         _ => None,
    //     }
    // }

    // pub fn len(&self) -> i32 {
    //     match self {
    //         Self::Undefined => i32::MAX,
    //         Self::Eof => 0,
    //         Self::Identifier(id) => id.len() as i32,
    //         Self::Number(num) => num.len() as i32,
    //         Self::True => 4,
    //         Self::False => 5,
    //         Self::Plus => 1,
    //         Self::Minus => 1,
    //         Self::Multiply => 1,
    //         Self::Divide => 1,
    //         Self::Assign => 1,
    //         Self::Bang => 1,
    //         Self::Equals => 2,
    //         Self::NotEqual => 2,
    //         Self::GreaterThan => 1,
    //         Self::GreaterEqualThan => 2,
    //         Self::LessThan => 1,
    //         Self::LessEqualThan => 2,
    //         Self::LeftParen => 1,
    //         Self::RightParen => 1,
    //         Self::Comma => 1,
    //         Self::SemiColon => 1,
    //         Self::LeftBrace => 1,
    //         Self::RightBrace => 1,
    //         Self::If => 2,
    //         Self::Else => 2,
    //         Self::While => 5,
    //         Self::Return => 6,
    //         Self::Break => 5,
    //         Self::Forward => 7,
    //         Self::Left => 4,
    //         Self::Right => 5,
    //         Self::Func => 4,
    //     }
    // }
}

#[derive(Debug, Clone, Copy)]
pub struct Loc {
    pub line: usize,
    pub col: usize,
    pub byte: usize,
    pub char: usize,
}

#[derive(Clone, Copy)]
pub struct Span<T> {
    value: T,
    from: Loc,
    to: Loc,
}

impl<T> Span<T> {
    pub fn value(self) -> T {
        self.value
    }

    pub fn from(self) -> Loc {
        self.from
    }

    pub fn to(self) -> Loc {
        self.to
    }
}

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    chars: Chars<'a>,
    current_char: Option<char>,
    current_loc: Loc,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let curr = chars.next();

        Lexer {
            input,
            chars,
            current_char: curr,
            current_loc: Loc {
                line: 0,
                col: 0,
                byte: 0,
                char: 0,
            },
        }
    }

    fn advance(&mut self) {
        if let Some(c) = self.current_char {
            self.current_loc.char += 1;
            self.current_loc.byte += c.len_utf8();

            if c == '\n' {
                self.current_loc.line += 1;
                self.current_loc.col = 0;
            } else {
                self.current_loc.col += 1;
            }
        }

        self.current_char = self.chars.next();
    }

    fn read_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if !c.is_whitespace() {
                break;
            }

            self.advance();
        }
    }

    /// Returns true when given `char` cannot be part of an identifier
    /// string.
    /// 
    fn is_forbidden_identifier_char(&self, char: char) -> bool {
        matches!(
            char,
            ',' | '.' | ';' | '(' | ')' | '-' | '+' | '/' | '*' | '^' | '='
        )
    }

    fn read_identifier(&mut self) -> &'a str {
        let start = self.current_loc.byte;

        while let Some(c) = self.current_char {
            if c.is_whitespace() || self.is_forbidden_identifier_char(c) {
                break;
            }

            self.advance();
        }

        &self.input[start..self.current_loc.byte]
    }

    fn read_number(&mut self) -> &'a str {
        let start = self.current_loc.byte;

        // Everything before the delimiter
        while self.current_char.map_or(false, |c| c.is_ascii_digit()) {
            self.advance()
        }

        // The delimiter
        if self.current_char.map_or(false, |c| c == '.') {
            self.advance()
        }

        // Everything after the delimiter
        while self.current_char.map_or(false, |c| c.is_ascii_digit()) {
            self.advance()
        }

        &self.input[start..self.current_loc.byte]
    }

    pub fn next_token(&mut self) -> Span<Token<'a>> {
        self.read_whitespace();

        let start = self.current_loc;

        if self.current_char.is_none() {
            return Span {
                value: Token::Eof,
                from: start,
                to: start,
            };
        }

        let token = match self.current_char.unwrap() {
            ';' => {
                self.advance();
                Token::SemiColon
            }
            '+' => {
                self.advance();
                Token::Plus
            }
            '-' => {
                self.advance();
                Token::Minus
            }
            '*' => {
                self.advance();
                Token::Multiply
            }
            '/' => {
                self.advance();
                Token::Divide
            }
            '=' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::Equals
                } else {
                    Token::Assign
                }
            }
            '!' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::NotEqual
                } else {
                    Token::Bang
                }
            }
            '>' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::GreaterEqualThan
                } else {
                    Token::GreaterThan
                }
            }
            '<' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::LessEqualThan
                } else {
                    Token::LessThan
                }
            }
            '(' => {
                self.advance();
                Token::LeftParen
            }
            ')' => {
                self.advance();
                Token::RightParen
            }
            '{' => {
                self.advance();
                Token::LeftBrace
            }
            '}' => {
                self.advance();
                Token::RightBrace
            }
            ',' => {
                self.advance();
                Token::Comma
            }
            'a'..='z' | 'A'..='Z' | '_' => match self.read_identifier() {
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
            '0'..='9' => {
                let num = self.read_number();
                Token::Number(num)
            }
            _ => {
                self.advance();
                Token::Undefined
            }
        };

        let end = self.current_loc;

        Span {
            value: token,
            from: start,
            to: end,
        }
    }

    pub fn tokens(&mut self) -> Vec<Span<Token<'a>>> {
        let mut vec = Vec::new();
        let mut token = self.next_token();

        while token.value != Token::Eof {
            vec.push(token);
            token = self.next_token();
        }

        vec.push(token);
        vec
    }
}

impl<T> Debug for Span<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}
