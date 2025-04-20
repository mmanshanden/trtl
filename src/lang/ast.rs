use core::fmt;
use std::{fmt::Debug, rc::Rc};

/// The `Token` type
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
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

    Whitespace(&'a str),
    Comment(&'a str),
    LineBreak
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier(id) => write!(f, "{}", id),
            Self::Number(num) => write!(f, "{}", num),
            Self::Whitespace(ws) => write!(f, " "),
            Self::Comment(comment) => fmt::Result::Ok(()),
            Self::LineBreak => write!(f, " "),
            Self::Undefined => write!(f, ""),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
            Self::Assign => write!(f, "="),
            Self::Bang => write!(f, "!"),
            Self::Equals => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterEqualThan => write!(f, ">="),
            Self::LessThan => write!(f, "<"),
            Self::LessEqualThan => write!(f, "<="),
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::Comma => write!(f, ","),
            Self::SemiColon => write!(f, ";"),
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::While => write!(f, "while"),
            Self::Return => write!(f, "return"),
            Self::Break => write!(f, "break"),
            Self::Forward => write!(f, "forward"),
            Self::Left => write!(f, "left"),
            Self::Right => write!(f, "right"),
            Self::Func => write!(f, "func"),
            Self::Eof => write!(f, "EOF"),
        }
    }
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

    fn read_identifier(&mut self) -> &'a str {
        let start = self.pos.byte;

        loop {
            if self.current_char.is_none() {
                break;
            }

            if self.is_whitespace() {
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
                self.advance();
                Token::Undefined
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


#[derive(Debug)]
pub enum Expr {
    Assign(Box<Expr>, Box<Expr>),

    Equals(Box<Expr>, Box<Expr>),
    NotEqual(Box<Expr>, Box<Expr>),
    GreaterThan(Box<Expr>, Box<Expr>),
    GreaterEqualThan(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    LessEqualThan(Box<Expr>, Box<Expr>),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    Var(String),
    Num(f64),
    Call(String, Vec<Expr>),
}

#[derive(Debug)]
pub enum Stmt {
    If(Expr, Box<Stmt>),
    IfElse(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
    Forward(Expr),
    Left(Expr),
    Right(Expr),
    Scope(Vec<Stmt>),
    Expr(Expr),
    Return(Option<Expr>)
}

#[derive(Debug)]
pub enum Entry {
    Func(String, Vec<String>, Stmt),
    Stmt(Stmt)
}

pub type Program = Vec<Entry>;


#[derive(Debug, Clone)]
pub enum Chain<T> {
    Nil,
    Cons(T, Rc<Chain<T>>)
}

pub type Deny<'a> = Vec<Token<'a>>;

impl<T> Chain<T> {
    pub fn new() -> Self {
        Self::Nil
    }

    fn push(self, elem: T) -> Self {
        match self {
            Self::Nil => {
                Self::Cons(elem, Rc::new(Self::Nil))
            }
            Self::Cons(head, tail) => {
                let new_tail = Self::Cons(head, tail);
                Self::Cons(elem, Rc::new(new_tail))
            }
        }
    }
}

impl<T> Chain<T> where T: PartialEq {
    fn contains(&self, elem: &T) -> bool {
        match self {
            Self::Nil => false,
            Self::Cons(t, next) => {
                if t == elem {
                    true
                } else {
                    next.contains(elem)
                }
            }
        }
    }
}


pub type Tokens<'a> = &'a [Token<'a>];


#[derive(Debug, Clone)]
pub enum Tag<'a> {
    Number(&'a str),
    Identifier(&'a str),
    Call(&'a str),
    Whitespace(Tokens<'a>),
    Keyword(Token<'a>),
    Plain(Token<'a>),
    Move(Token<'a>),

    UnexpectedToken {
        expected: Token<'a>,
        actual: Tokens<'a>
    }
}

trait Contains<'a> {
    fn contains(&self, token: &'a Token<'a>) -> bool;
}

impl<'a> Contains<'a> for Deny<'a> {
    fn contains(&self, token: &'a Token<'a>) -> bool {
        self.iter().any(|t| t == token)
    }
}

impl<'a, F> Contains<'a> for F
where
    F: Fn(&'a Token<'a>) -> bool,
{
    fn contains(&self, token: &'a Token<'a>) -> bool {
        self(token)
    }
}

#[derive(Clone)]
pub struct Run<'a> {
    dist: usize,
    input: Tokens<'a>,
}

impl Debug for Run<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let input_concatenated = self.input.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>().join("");
        write!(f, "{{ dist: {}, input: {:?} }}", self.dist, input_concatenated)
    }
}

impl<'a> Run<'a> {
    pub fn new(input: Tokens<'a>) -> Self {
        Run {
            dist: 0,
            input,
        }
    }

    fn next(&self, deny: impl Contains<'a>) -> Option<(usize, Tokens<'a>, Token<'a>, Run<'a>)> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if deny.contains(token) {
                let next = Run {
                    dist: self.dist + dist,
                    input: &self.input[idx + 1..],
                };

                return Some((
                    dist,
                    &self.input[..idx],
                    *token,
                    next,
                ));
            }

            dist += token.dist();
        }

        None
    }

    fn first_where(
        &self,
        pred: impl Fn(&'a Token<'a>) -> bool,
        deny: impl Contains<'a>,
    ) -> Option<(usize, Tokens<'a>, Token<'a>, Run<'a>)> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if pred(token) {
                let next = Run {
                    dist: self.dist + dist,
                    input: &self.input[idx + 1..],
                };

                return Some((
                    dist,
                    &self.input[..idx],
                    *token,
                    next,
                ));
            }

            if deny.contains(token) {
                return None;
            }

            dist += token.dist();
        }

        None
    }

    fn first_where_some<T>(
        &self,
        pred: impl Fn(&'a Token<'a>) -> Option<T>,
        deny: impl Contains<'a>,
    ) -> Option<(usize, T, Tokens<'a>, Token<'a>, Run<'a>)> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if let Some(value) = pred(token) {
                let next = Run {
                    dist: self.dist + dist,
                    input: &self.input[idx + 1..],
                };

                return Some((
                    dist,
                    value,
                    &self.input[..idx],
                    *token,
                    next,
                ));
            }

            if deny.contains(token) {
                return None;
            }

            dist += token.dist();
        }

        None
    }
}


type Tags<'a> = Vec<Tag<'a>>;

#[derive(Debug, Clone)]
pub enum ParseResult<'a, T> {
    Success(T, Tags<'a>, Run<'a>),
    Error(Run<'a>),
}

impl<'a, T> ParseResult<'a, T> {
    pub fn is_ok(&self) -> bool {
        match self {
            Self::Error(_) => false,
            _ => true,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            Self::Error(_) => true,
            _ => false,
        }
    }

    pub fn unwrap(self) -> (T, Tags<'a>, Run<'a>) {
        match self {
            Self::Success(value, fragments, run) => (value, fragments, run),
            Self::Error(_) => panic!("Called `unwrap` on an `Error` value"),
        }
    }

    pub fn dist(&self) -> usize {
        match self {
            Self::Success(_, _, run) => run.dist,
            Self::Error(run) => panic!("Called `dist` on an `Error` value"),
        }
    }

    pub fn remaining_input_len(&self) -> usize {
        match self {
            Self::Success(_, _, run) => run.input.len(),
            Self::Error(run) => run.input.len()
        }
    }

    pub fn map<U>(self, map: impl Fn(T) -> U) -> ParseResult<'a, U> {
        match self {
            Self::Success(value, fragments, run) => {
                let mapped_value = map(value);
                ParseResult::Success(mapped_value, fragments, run)
            }
            Self::Error(run) => ParseResult::Error(run),
        }
    }
}


pub trait Parser<'a, T> {
    fn parse(&self, run: Run<'a>, deny: Deny<'a>) -> ParseResult<'a, T>;

    fn map<U>(self, map: impl Fn(T) -> U) -> impl Parser<'a, U>
    where
        Self: Sized 
    {
        move |run: Run<'a>, deny: Deny<'a>| {
            let result = self.parse(run, deny);

            let (value, tags, run) = match result {
                ParseResult::Success(value, tags, run) => (value, tags, run),
                ParseResult::Error(run) => return ParseResult::Error(run),
            };

            let mapped_value = map(value);

            ParseResult::Success(mapped_value, tags, run)
        }
    }

    fn optional(self) -> impl Parser<'a, Option<T>> 
    where
        Self: Sized 
    {
        move |run: Run<'a>, deny: Deny<'a>| {
            match self.parse(run, deny) {
                ParseResult::Success(value, tags, run) => {
                    ParseResult::Success(Some(value), tags, run)
                }
                ParseResult::Error(run) => {
                    ParseResult::Success(None, vec![], run)
                }
            }
        }
    }

    fn deny(self, deny: Deny<'a>) -> impl Parser<'a, T> 
    where
        Self: Sized 
    {
        move |run: Run<'a>, parent: Deny<'a>| {
            self.parse(run, [deny.clone(), parent].concat())
        }
    }

    fn debug(self, name: &'static str) -> impl Parser<'a, T>
    where 
        Self: Sized
    {
        move |run: Run<'a>, deny: Deny<'a>| {

            println!("[{}] [RUN] {:?} deny: {:?}", name, run, deny);
            
            let result = self.parse(run, deny);

            if result.is_err() {
                println!("\t[{}] [ERR]", name);
                result
            } else {
                let (value, tags, run) = result.unwrap();
                println!("\t[{}] [OK] {:?}", name, run);
                ParseResult::Success(value, tags, run)
            }
        }
    }
}

impl<'a, T, F> Parser<'a, T> for F
where   
    F: Fn(Run<'a>, Deny<'a>) -> ParseResult<'a, T>
{
    fn parse(&self, run: Run<'a>, deny: Deny<'a>) -> ParseResult<'a, T> {
        self(run, deny)
    }
}

fn expect<'a, U>(expect: Token<'static>, detoken: U) -> impl Parser<'a, ()>
where
    U: Fn(Token<'a>) -> Tag<'a>,
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let next = run.first_where(|&t| t == expect, deny);

        if next.is_none() {
            return ParseResult::Error(run);
        }

        let (_, tokens, token, run) = next.unwrap();

        let tags = if tokens.is_empty() { 
            vec![detoken(token)] 
        } else if tokens.iter().map(Token::dist).sum::<usize>() == 0 {
            vec![Tag::Whitespace(tokens), detoken(token)]
        } else {
            vec![
                Tag::UnexpectedToken {
                    expected: token,
                    actual: tokens,
                }, 
                detoken(token)
            ]
        };

        ParseResult::Success((), tags, run)
    }
}


fn expect_next<'a, U>(expect: Token<'static>, detoken: U) -> impl Parser<'a, ()>
where
    U: Fn(Token<'a>) -> Tag<'a>
{
    move |run: Run<'a>, _: Deny<'a>| {
        let next = run.next(|t: &Token| t.dist() > 0);

        if next.is_none() {
            return ParseResult::Error(run);
        }

        let (_, tokens, token, run) = next.unwrap();

        if token != expect {
            return ParseResult::Error(run);
        }

        let fragments = if tokens.is_empty() { 
            vec![detoken(expect)] 
        } else { 
            vec![Tag::Whitespace(tokens), detoken(expect)] 
        };

        ParseResult::Success((), fragments, run)
    }
}

// fn expect_forced<'a, U>(expect: Token<'static>, detoken: U) -> impl Parser<'a, ()>
// where
//     U: Fn(Token<'a>) -> Tag<'a>,
// {
//     move |run: Run<'a>, deny: Deny<'a>| {
//         let result = run.first_where(|&t: &'a Token| t == expect, |&t: &'a Token| {
//             t != expect && deny.contains(&t)
//         });

//         if result.is_none() {
//             return ParseResult::Error(run);
//         }

//         let (_, tokens, token, run) = result.unwrap();

//         let tags = if tokens.is_empty() { 
//             vec![Tag::Plain(token)] 
//         } else if tokens.iter().map(Token::dist).sum::<usize>() == 0 {
//             vec![Tag::Whitespace(tokens), detoken(token)]
//         } else {
//             vec![
//                 Tag::UnexpectedToken {
//                     expected: token,
//                     actual: tokens,
//                 }, 
//                 detoken(token)
//             ]
//         };

//         ParseResult::Success((), tags, run)
//     }
// }

fn expect_some<'a, F, U, T>(expect: F, detoken: U) -> impl Parser<'a, T>
where
    T: Copy,
    F: Fn(&'a Token<'a>) -> Option<T>,
    U: Fn(T, Token<'a>) -> Tag<'a>,
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let next = run.first_where_some(&expect, deny);

        if next.is_none() {
            return ParseResult::Error(run);
        }

        let (dist, val, tokens, token, run) = next.unwrap();

        let tags = if tokens.is_empty() { 
            vec![detoken(val, token)] 
        } else if dist == 0 {
            vec![Tag::Whitespace(tokens), detoken(val, token)]
        } else {
            vec![
                Tag::UnexpectedToken {
                    expected: token,
                    actual: tokens,
                }, 
                detoken(val, token)
            ]
        };

        ParseResult::Success(val, tags, run)
    }
}

fn choice<'a, P, T>(parsers: Vec<P>) -> impl Parser<'a, T> 
where 
    P: Parser<'a, T> 
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let mut best = ParseResult::Error(run.clone());

        for parser in &parsers {
            let inner_run = run.clone();
            let inner_deny = deny.clone();
            let result = parser.parse(inner_run, inner_deny);

            if result.is_err() {
                continue;
            }

            let (value, tags, next) = result.unwrap();

            if next.dist == run.dist {
                best = ParseResult::Success(value, tags, next);
                break;
            }

            if best.is_err() { 
                best = ParseResult::Success(value, tags, next);
                continue;
            }

            if next.dist < best.dist() {
                best = ParseResult::Success(value, tags, next);
                continue;
            }

            if next.input.len() < best.remaining_input_len() {
                best = ParseResult::Success(value, tags, next);
                continue;
            }
        }

        best
    }
}

fn combine<'a, A, B, P, Q, T, F>(first: P, second: Q, combine: F) -> impl Parser<'a, T> 
where
    P: Parser<'a, A>,
    Q: Parser<'a, B>,
    F: Fn(A, B) -> T,
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let result1 = first.parse(run.clone(), deny.clone());

        if result1.is_err() {
            return ParseResult::Error(run);
        }

        let (value1, tags1, run) = result1.unwrap();
        let result2 = second.parse(run.clone(), deny.clone());

        if result2.is_err() {
            return ParseResult::Error(run);
        }

        let (value2, tags2, run) = result2.unwrap();

        let tags3 = [tags1, tags2].concat();
        let result3 = combine(value1, value2);

        ParseResult::Success(result3, tags3, run)
    }
}

fn many<'a, P, T>(parser: P) -> impl Parser<'a, Vec<T>> 
where
    P: Parser<'a, T>,
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let mut run = run;
        let mut tags = Vec::new();
        let mut values = Vec::new();

        loop {
            let inner_run = run.clone();
            let inner_deny = deny.clone();
            let result = parser.parse(inner_run, inner_deny);

            if result.is_err() {
                break;
            }

            let (value, mut ts, next) = result.unwrap();

            values.push(value);
            tags.append(&mut ts);
            
            run = next;
        }

        ParseResult::Success(values, tags, run)
    }
}

fn wrapped<'a, T>(before: Token<'static>, after: Token<'static>, body: impl Parser<'a, T>) -> impl Parser<'a, T> {
    combine(
        expect(before, Tag::Plain),
        combine(
            body.deny(vec![after]),
            expect(after, Tag::Plain),
            |body, _| body
        ),
        |_, body| body
    )
}

fn wrapped_attached<'a, T>(before: Token<'static>, after: Token<'static>, body: impl Parser<'a, T>) -> impl Parser<'a, T> {
    combine(
        expect_next(before, Tag::Plain),
        combine(
            body.deny(vec![after]),
            expect(after, Tag::Plain),
            |body, _| body
        ),
        |_, body| body
    )
}


fn seperated_by<'a, T>(parser: impl Parser<'a, T>, separator: Token<'static>) -> impl Parser<'a, Vec<T>> {
    let parser = parser.deny(vec![separator]);

    move |run: Run<'a>, deny: Deny<'a>| {

        // parse the first element
        let first_element = parser.parse(run.clone(), deny.clone());

        let (mut values, mut tags, mut run) = match first_element {
            ParseResult::Error(run) => {
                return ParseResult::Success(Vec::new(), Vec::new(), run)
            },
            ParseResult::Success(value, tags, run) => {
                (vec![value], tags, run)
            }
        };

        loop {
            // parse the separator
            match expect(separator, Tag::Plain).parse(run.clone(), deny.clone()) {
                ParseResult::Error(_) => { 
                    break;
                },
                ParseResult::Success(_, mut ts, next) => {
                    tags.append(&mut ts);
                    run = next;
                }
            };

            // parse the next element
            match parser.parse(run.clone(), deny.clone()) {
                ParseResult::Error(_) => { 
                    break;
                },
                ParseResult::Success(value, mut ts, next) => {
                    values.push(value);
                    tags.append(&mut ts);
                    run = next;
                }
            }
        }

        ParseResult::Success(values, tags, run)
    }
}

pub fn parse_expr_primary<'a>() -> impl Parser<'a, Expr> {
    let number_parser = |run: Run<'a>, deny: Deny<'a>| {
        expect_some(
            Token::number,
            |number, _| Tag::Number(number)
        ).map(|number| {
            let number = number.parse().unwrap();
            Expr::Num(number)
        }).parse(run, deny)
    };

    let func_call_parser = |run: Run<'a>, deny: Deny<'a>| {
        combine(
            expect_some(Token::identifier, |id, _| Tag::Call(id)),
            wrapped_attached(
                Token::LeftParen,
                Token::RightParen,
                seperated_by(parse_expr(), Token::Comma)
            ),
            |id, args| {
                let id = id.to_string();
                Expr::Call(id, args)
            }
        ).parse(run, deny)
    };

    let identifier_parser = |run: Run<'a>, deny: Deny<'a>| {
        expect_some(
            Token::identifier,
            |id, _| Tag::Identifier(id)
        ).map(|id| {
            let id = id.to_string();
            Expr::Var(id)
        }).parse(run, deny)
    };

    let sub_expr_parser = |run: Run<'a>, deny: Deny<'a>| {
        wrapped(
            Token::LeftParen, 
            Token::RightParen, 
            parse_expr()
        ).parse(run, deny)
    };

    choice(vec![
        number_parser,
        func_call_parser,
        identifier_parser,
        sub_expr_parser,
    ])
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Associativity {
    Left,
    Right,
}


/// The recurrent case for a given minimum precedence level
/// 
/// See: https://eli.thegreenplace.net/2012/08/02/parsing-expressions-by-precedence-climbing
///
fn parse_expr_1<'a>(min_prec: u8) -> impl Parser<'a, Expr>{
    move |run: Run<'a>, deny: Deny<'a>| {
        let mut tags = Vec::new();
        let mut run = run;

        let mut lhs = match parse_expr_primary().parse(run, deny.clone()) {
            ParseResult::Error(run) => return ParseResult::Error(run),
            ParseResult::Success(expr, mut fs, next) => {
                tags.append(&mut fs);
                run = next;
                expr
            }
        };

        let is_operation = |&token| match token {
            Token::Equals => Some((Token::Equals, 1, Associativity::Left)),
            Token::NotEqual => Some((Token::NotEqual, 1, Associativity::Left)),
            Token::LessThan => Some((Token::LessThan, 1, Associativity::Left)),
            Token::LessEqualThan => Some((Token::LessEqualThan, 1, Associativity::Left)),
            Token::GreaterThan => Some((Token::GreaterThan, 1, Associativity::Left)),
            Token::GreaterEqualThan => Some((Token::GreaterEqualThan, 1, Associativity::Left)),
            Token::Plus => Some((Token::Plus, 2, Associativity::Left)),
            Token::Minus => Some((Token::Minus, 2, Associativity::Left)),
            Token::Multiply => Some((Token::Multiply, 3, Associativity::Left)),
            Token::Divide => Some((Token::Divide, 3, Associativity::Left)),
            _ => None,
        };

        loop {
            let inner_run = run.clone();
            let inner_deny = deny.clone();

            let (op, prec, assoc) = match expect_some(is_operation, |_, token| Tag::Plain(token)).parse(inner_run, inner_deny) {
                ParseResult::Error(_) => break,
                ParseResult::Success(some, mut fs, next) => {
                    tags.append(&mut fs);
                    run = next;
                    some
                }
            };

            let new_min_prec = if assoc == Associativity::Left {
                prec + 1
            } else {
                prec
            };

            let rhs = match parse_expr_1(new_min_prec).parse(run.clone(), deny.clone()) {
                ParseResult::Error(run) => return ParseResult::Error(run),
                ParseResult::Success(rhs, mut fs, next) => {
                    tags.append(&mut fs);
                    run = next;
                    rhs
                }
            };

            let lhs_boxxed = Box::new(lhs);
            let rhs_boxxed = Box::new(rhs);

            lhs = match op {
                Token::Assign => Expr::Assign(lhs_boxxed, rhs_boxxed),
                Token::Equals => Expr::Equals(lhs_boxxed, rhs_boxxed),
                Token::NotEqual => Expr::NotEqual(lhs_boxxed, rhs_boxxed),
                Token::GreaterThan => Expr::GreaterThan(lhs_boxxed, rhs_boxxed),
                Token::GreaterEqualThan => Expr::GreaterEqualThan(lhs_boxxed, rhs_boxxed),
                Token::LessThan => Expr::LessThan(lhs_boxxed, rhs_boxxed),
                Token::LessEqualThan => Expr::LessEqualThan(lhs_boxxed, rhs_boxxed),
                Token::Plus => Expr::Add(lhs_boxxed, rhs_boxxed),
                Token::Minus => Expr::Sub(lhs_boxxed, rhs_boxxed),
                Token::Multiply => Expr::Mul(lhs_boxxed, rhs_boxxed),
                Token::Divide => Expr::Div(lhs_boxxed, rhs_boxxed),
                _ => unreachable!(),
            };
        }

        ParseResult::Success(lhs, tags, run)
    }
}

pub fn parse_expr<'a>() -> impl Parser<'a, Expr> {
    parse_expr_1(0)
}

fn parse_stmt_block<'a>() -> impl Parser<'a, Stmt> {
    let block_parser = |run: Run<'a>, deny: Deny<'a>| {
        wrapped(
            Token::LeftBrace,
            Token::RightBrace,
            many(parse_stmt())
        ).map(Stmt::Scope).parse(run, deny)
    };

    block_parser
}

fn parse_stmt_expr<'a>() -> impl Parser<'a, Stmt> {
    combine(
        parse_expr().deny(vec![Token::SemiColon]),
        expect(Token::SemiColon, Tag::Plain),
        |expr, _| Stmt::Expr(expr)
    )
}

pub fn parse_stmt_move<'a>() -> impl Parser<'a, Stmt> {
    combine(
        expect_some(|&token| match token {
            Token::Forward => Some(Token::Forward),
            Token::Left => Some(Token::Left),
            Token::Right => Some(Token::Right),
            _ => None,
        }, |_, token| Tag::Move(token)),
        combine(
            parse_expr().deny(vec![Token::SemiColon]),
            expect(Token::SemiColon, Tag::Plain),
            |expr, _| expr
        ),
        |stmt, expr| match stmt {
            Token::Forward => Stmt::Forward(expr),
            Token::Left => Stmt::Left(expr),
            Token::Right => Stmt::Right(expr),
            _ => unreachable!(),
        }
    )
}

pub fn parse_stmt_if<'a>() -> impl Parser<'a, Stmt> {
    let parse_if = combine(
        expect(Token::If, Tag::Keyword),
        combine(
            parse_expr().deny(vec![Token::LeftBrace]),
            parse_stmt_block().deny(vec![Token::Else]),
            |expr, stmt| (expr, stmt)
        ),
        |_, if_block| if_block
    );

    let parse_else = |run: Run<'a>, deny: Deny<'a>| {
        let r = run.first_where(|&t| t == Token::Else, deny.clone());

        if r.is_none() {    
            return ParseResult::Success(None, vec![], run)
        }
        
        let (dist, tokens, _, next) = r.unwrap();

        if dist > 0 {
            return ParseResult::Success(None, vec![], run)
        }

        let tags = if tokens.is_empty() {
            vec![Tag::Keyword(Token::Else)]
        } else {
            vec![Tag::Whitespace(tokens), Tag::Keyword(Token::Else)]
        };

        match parse_stmt_block().parse(next, deny) {
            ParseResult::Error(run) => {
                ParseResult::Error(run)
            },
            ParseResult::Success(stmt, inner, next) => {
                ParseResult::Success(Some(stmt), [tags, inner].concat(), next)
            }
        }
    };

    combine(
        parse_if,
        parse_else,
        |if_block, else_block| {
            let (if_expr, if_body) = if_block;
            if let Some(else_body) = else_block {
                Stmt::IfElse(if_expr, Box::new(if_body), Box::new(else_body))
            } else {
                Stmt::If(if_expr, Box::new(if_body))
            }
        }
    )
}

fn parse_stmt<'a>() -> impl Parser<'a, Stmt> {
    let expr_parser = |run: Run<'a>, deny: Deny<'a>| {
        parse_stmt_expr().parse(run, deny)
    };

    let if_parser = |run: Run<'a>, deny: Deny<'a>| {
        parse_stmt_if().parse(run, deny)
    };

    let move_parser = |run: Run<'a>, deny: Deny<'a>| {
        parse_stmt_move().parse(run, deny)
    };

    choice(vec![
        expr_parser,
        if_parser,
        move_parser,
    ])
}

pub fn parse_func<'a>() -> impl Parser<'a, Entry> {
    let arg_parser = expect_some(
        Token::identifier,
        |id, _| Tag::Identifier(id)
    );

    let params_parser = wrapped_attached(
        Token::LeftParen, 
        Token::RightParen,
        seperated_by(arg_parser, Token::Comma)
    );

    let header_parser = combine(
        expect(Token::Func, Tag::Keyword),
        combine(
            expect_some(
                Token::identifier,
                |id, _| Tag::Identifier(id)
            ),
            params_parser,
            |name, params| (name, params)
        ),
        |_, header| header
    );

    combine(
        header_parser,
        parse_stmt_block(),
        |header, body| {
            let (name, params) = header;
            let params = params.iter().map(|&s| s.to_string()).collect();
            Entry::Func(name.to_string(), params, body)
        }
    )
}

pub fn parse_program<'a>() -> impl Parser<'a, Program> {
    let parse_stmt = |run: Run<'a>, deny: Deny<'a>| {
        parse_stmt().parse(run, deny).map(Entry::Stmt)
    };

    let parse_func = |run: Run<'a>, deny: Deny<'a>| {
        parse_func().parse(run, deny)
    };

    combine(
        many(
            choice(vec![
                parse_stmt,
                parse_func,
            ]),
        ),
        expect(Token::Eof, Tag::Plain),
        |entries, _| entries
    )
}
