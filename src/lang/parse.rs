use std::rc::Rc;

use super::{
    Marker, Markers,
    ast::{Entry, Expr, Program, Stmt},
    lex::{Symbol, Tokens},
    run::{Contains, ReadResult, Run},
};

#[derive(Debug, Clone)]
pub enum Deny<'a> {
    Cons(Symbol<'a>, Rc<Deny<'a>>),
    Nil,
}

impl<'a> Deny<'a> {
    pub fn new() -> Self {
        Deny::Nil
    }

    pub fn insert(&self, token: Symbol<'a>) -> Self {
        let clone = self.clone();
        Self::Cons(token, Rc::new(clone))
    }
}

impl<'a> Contains<'a> for Deny<'a> {
    fn contains(&self, token: &'a Symbol<'a>) -> bool {
        match self {
            Deny::Cons(value, tail) => token == value || tail.contains(token),
            Deny::Nil => false,
        }
    }
}

impl<'a> Contains<'a> for &Deny<'a> {
    fn contains(&self, token: &'a Symbol<'a>) -> bool {
        match self {
            Deny::Cons(value, tail) => token == value || tail.contains(token),
            Deny::Nil => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParseResult<'a, T> {
    Success(T, Run<'a>),
    Error,
}

impl<'a, T> ParseResult<'a, T> {
    pub fn is_ok(&self) -> bool {
        match self {
            Self::Error => false,
            _ => true,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            Self::Error => true,
            _ => false,
        }
    }

    pub fn unwrap(self) -> (T, Run<'a>) {
        match self {
            Self::Success(value, run) => (value, run),
            Self::Error => panic!("Called `unwrap` on an `Error` value"),
        }
    }

    pub fn dist(&self) -> usize {
        match self {
            Self::Success(_, run) => run.dist(),
            Self::Error => panic!("Called `dist` on an `Error` value"),
        }
    }

    pub fn remaining_input_len(&self) -> usize {
        match self {
            Self::Success(_, run) => run.remaining_input_len(),
            Self::Error => panic!("Called `remaining_input_len` on an `Error` value"),
        }
    }

    pub fn map<U>(self, map: impl Fn(T) -> U) -> ParseResult<'a, U> {
        match self {
            Self::Success(value, run) => ParseResult::Success(map(value), run),
            Self::Error => ParseResult::Error,
        }
    }
}
