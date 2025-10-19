use std::rc::Rc;

use crate::lang::Symbol;

use super::lex::{Token, Tokens};

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

pub trait Contains<'a> {
    fn contains(&self, token: &'a Symbol<'a>) -> bool;
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

impl<'a, F> Contains<'a> for F
where
    F: Fn(&'a Symbol<'a>) -> bool,
{
    fn contains(&self, symbol: &'a Symbol<'a>) -> bool {
        self(symbol)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Range {
    index_from: usize,
    index_to: usize,
}

impl Range {
    pub fn new(index_from: usize, index_to: usize) -> Self {
        Range {
            index_from,
            index_to,
        }
    }

    pub fn len(&self) -> usize {
        self.index_to - self.index_from
    }

    pub fn extend(self, other: Range) -> Self {
        Range {
            index_from: self.index_from,
            index_to: other.index_to,
        }
    }
}

#[derive(Debug)]
pub struct Span<'a> {
    dist: usize,
    source: Run<'a>,
    range: Range,
}

/// A span represents a contiguous sequence of one or more tokens read from the
/// input stream by a Run instance, ending at the token that satisfied the read
/// condition.
///
impl<'a> Span<'a> {
    pub fn all(&self) -> Tokens<'a> {
        &self.source.tokens[..self.range.index_from]
    }

    pub fn init(&self) -> Tokens<'a> {
        &self.source.tokens[..self.range.index_from - 1]
    }

    pub fn last(&self) -> Token<'a> {
        self.source.tokens[self.range.index_to - 1]
    }

    pub fn peek_symbol(&self) -> Option<&'a Symbol<'a>> {
        self.source
            .tokens
            .get(self.range.index_to)
            .map(|token| &token.symbol)
    }

    pub fn range(&self) -> Range {
        self.range
    }

    pub fn revert(self) -> Run<'a> {
        self.source
    }

    pub fn cont(self) -> Run<'a> {
        Run {
            dist: self.source.dist + self.dist,
            tokens: &self.source.tokens,
            position: self.range.index_to,
        }
    }
}

pub enum ReadResult<'a, T> {
    Some(T, Span<'a>),
    None(Run<'a>),
}

impl<'a, T> ReadResult<'a, T> {
    pub fn unwrap_run(self) -> Run<'a> {
        match self {
            ReadResult::Some(_, span) => span.cont(),
            _ => panic!("Called `unwrap_run` on a `ReadResult` that is not `Some`."),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Run<'a> {
    /// A penalty score representing the number of skipped tokens
    dist: usize,

    /// All tokens in the input stream
    tokens: Tokens<'a>,

    /// Current position in the token stream
    position: usize,
}

impl<'a> Run<'a> {
    pub fn new(input: Tokens<'a>) -> Self {
        Run {
            dist: 0,
            tokens: input,
            position: 0,
        }
    }

    fn stream(&self) -> Tokens<'a> {
        &self.tokens[self.position..]
    }

    pub fn dist(&self) -> usize {
        self.dist
    }

    pub fn remaining_input_len(&self) -> usize {
        self.tokens.len() - self.position
    }

    /// Reads the next token from the input stream.
    ///
    /// When a token is successfully read, a `ReadResult::Some` variant is returned,
    /// containing a unit value `()` and a `Span` covering the read token.
    ///
    /// If there are no more tokens to read, a `ReadResult::None` variant is returned.
    pub fn read_next(self) -> ReadResult<'a, ()> {
        if let Some(token) = self.stream().first() {
            ReadResult::Some(
                (),
                Span {
                    dist: token.dist(),
                    range: Range {
                        index_from: self.position,
                        index_to: self.position + 1,
                    },
                    source: self,
                },
            )
        } else {
            ReadResult::None(self)
        }
    }

    pub fn read_next_where(
        self,
        pred: impl Fn(&'a Symbol<'a>) -> bool,
        deny: impl Contains<'a>,
    ) -> ReadResult<'a, Token<'a>> {
        let mut dist = 0;

        for (idx, token) in self.stream().iter().enumerate() {
            if pred(&token.symbol) {
                return ReadResult::Some(
                    *token,
                    Span {
                        dist,
                        range: Range {
                            index_from: self.position,
                            index_to: self.position + idx,
                        },
                        source: self,
                    },
                );
            }

            if deny.contains(&token.symbol) {
                return ReadResult::None(self);
            }

            dist += token.dist();
        }

        ReadResult::None(self)
    }

    pub fn read_next_where_some<T>(
        self,
        pred: impl Fn(&'a Symbol<'a>) -> Option<T>,
        deny: impl Contains<'a>,
    ) -> ReadResult<'a, T> {
        let mut dist = 0;

        for (idx, token) in self.stream().iter().enumerate() {
            if let Some(value) = pred(&token.symbol) {
                return ReadResult::Some(
                    value,
                    Span {
                        dist: dist,
                        range: Range {
                            index_from: self.position,
                            index_to: self.position + idx,
                        },
                        source: self,
                    },
                );
            }

            if deny.contains(&token.symbol) {
                return ReadResult::None(self);
            }

            dist += token.dist();
        }

        ReadResult::None(self)
    }
}
