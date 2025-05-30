use std::{collections::{HashMap, HashSet}, fmt::Debug};

use super::{lex::Tokens, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(String, Box<Expr>),

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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Entry {
    Func(String, Vec<String>, Stmt),
    Stmt(Stmt)
}

pub type Program = Vec<Entry>;

#[derive(Debug, Clone, Copy)]
pub enum Scope {
    Global,
    Local
}

#[derive(Debug, Clone)]
pub struct Context {
    pub variables: HashMap<String, Scope>,
    pub functions: HashSet<(String, usize)>,
    pub scope: Scope
}

impl Context {
    pub fn new() -> Context {
        return Self {
            variables: HashMap::new(),
            functions: HashSet::new(),
            scope: Scope::Global
        };
    }
}


#[derive(Debug, Clone)]
pub enum Marker<'a> {
    Number(&'a str),
    Identifier(&'a str),
    Function(&'a str),
    Flow(Token<'a>),
    Call(&'a str),
    Keyword(Token<'a>),
    Plain(Token<'a>),
    Move(Token<'a>),
    Comment(&'a str),
    Whitespace(&'a str),
    LineBreak,

    UnexpectedToken {
        expected: Token<'a>,
        actual: Tokens<'a>,
    }
}

pub type Markers<'a> = Vec<Marker<'a>>;
