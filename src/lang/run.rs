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
        Self::Cons(token, Rc::new(self.clone()))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
        &self.source.tokens[..self.range.index_to]
    }

    pub fn init(&self) -> Tokens<'a> {
        &self.source.tokens[..self.range.index_to - 1]
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

    pub fn is_some(&self) -> bool {
        matches!(self, ReadResult::Some(_, _))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, ReadResult::None(_))
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
    /// containing the next `Symbol` in the stream and a `Span` covering the reading.
    ///
    /// If there are no more tokens to read, a `ReadResult::None` variant is returned.
    pub fn consume_next(self) -> ReadResult<'a, Token<'a>> {
        if let Some(token) = self.stream().first() {
            ReadResult::Some(
                *token,
                Span {
                    dist: 0,
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

    pub fn read_until_true(
        self,
        pred: impl Fn(&Symbol<'a>) -> bool,
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
                            index_to: self.position + idx + 1,
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

    // TODO: rename to read_until_some
    pub fn read_until_some<T>(
        self,
        pred: impl Fn(&Symbol<'a>) -> Option<T>,
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
                            index_to: self.position + idx + 1,
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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test token
    fn make_token(symbol: Symbol<'_>, n: usize) -> Token<'_> {
        Token {
            symbol,
            index_from: n - 1,
            index_to: n,
        }
    }

    #[test]
    fn test_run_new() {
        let tokens = &[make_token(Symbol::Identifier("test"), 1)];
        let run = Run::new(tokens);

        assert_eq!(run.dist, 0);
        assert_eq!(run.position, 0);
        assert_eq!(run.tokens.len(), 1);
    }

    #[test]
    fn test_run_consume_next1() {
        let tokens: &[Token] = &[];
        let run = Run::new(tokens);

        match run.consume_next() {
            ReadResult::Some(_, _) => panic!("Expected None, got Some"),
            ReadResult::None(_) => (),
        }
    }

    #[test]
    fn test_run_consume_next2() {
        let tokens = &[
            make_token(Symbol::Identifier("first"), 1),
            make_token(Symbol::Identifier("second"), 2),
        ];
        let run = Run::new(tokens);

        match run.consume_next() {
            ReadResult::Some(_, span) => {
                assert_eq!(span.dist, 0);
                assert_eq!(span.range.index_from, 0);
                assert_eq!(span.range.index_to, 1);
                assert_eq!(span.init().len(), 0);
                assert_eq!(span.last().symbol, Symbol::Identifier("first"));
                assert_eq!(span.last().index_from, 0);
                assert_eq!(span.last().index_to, 1);
            }
            ReadResult::None(_) => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_run_read_until_true1() {
        let tokens = &[
            make_token(Symbol::Identifier("target"), 1),
            make_token(Symbol::Identifier("other"), 2),
        ];
        let run = Run::new(tokens);
        let deny = Deny::new();

        let (token, span) = match run.read_until_true(|&s| s == Symbol::Identifier("target"), deny)
        {
            ReadResult::Some(token, span) => (token, span),
            ReadResult::None(_) => panic!("Expected Some, got None"),
        };

        assert_eq!(token.symbol, Symbol::Identifier("target"));
        assert_eq!(span.last(), token);
        assert_eq!(span.dist, 0);
        assert_eq!(span.range.index_from, 0);
        assert_eq!(span.range.index_to, 1);
        assert_eq!(span.init().len(), 0);

        match span.cont().consume_next() {
            ReadResult::Some(token, span) => {
                assert_eq!(span.dist, 0);
                assert_eq!(span.range.index_from, 1);
                assert_eq!(span.range.index_to, 2);
                assert_eq!(token.symbol, Symbol::Identifier("other"));
            }
            ReadResult::None(_) => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_run_read_until_true2() {
        let tokens = &[
            make_token(Symbol::Identifier("first"), 1),
            make_token(Symbol::Identifier("target"), 2),
            make_token(Symbol::Identifier("last"), 3),
        ];
        let run = Run::new(tokens);
        let deny = Deny::new();

        let (token, span) = match run.read_until_true(|s| s == &Symbol::Identifier("target"), deny)
        {
            ReadResult::Some(token, span) => (token, span),
            ReadResult::None(_) => panic!("Expected Some, got None"),
        };

        assert_eq!(token.symbol, Symbol::Identifier("target"));
        assert_eq!(span.last(), token);
        assert_eq!(span.dist, 1);
        assert_eq!(span.range.index_from, 0);
        assert_eq!(span.range.index_to, 2);
        assert_eq!(span.init().len(), 1);

        let first_init_token = span.init().get(0).unwrap();

        assert_eq!(first_init_token.symbol, Symbol::Identifier("first"));
        assert_eq!(first_init_token.index_from, 0);
        assert_eq!(first_init_token.index_to, 1);

        match span.cont().consume_next() {
            ReadResult::Some(token, span) => {
                assert_eq!(span.dist, 0);
                assert_eq!(span.range.index_from, 2);
                assert_eq!(span.range.index_to, 3);
                assert_eq!(token.symbol, Symbol::Identifier("last"));
            }
            ReadResult::None(_) => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_run_read_until_true3() {
        let tokens = &[
            make_token(Symbol::Identifier("first"), 1),
            make_token(Symbol::Identifier("second"), 2),
            make_token(Symbol::Identifier("target"), 3),
        ];
        let run = Run::new(tokens);
        let deny = Deny::new().insert(Symbol::Identifier("second"));

        assert_eq!(run.dist, 0);
        assert_eq!(run.position, 0);
        assert_eq!(run.tokens.len(), 3);

        let run = match run.read_until_true(|s| s == &Symbol::Identifier("target"), deny) {
            ReadResult::Some(_, _) => panic!("Expected None, got Some"),
            ReadResult::None(run) => run,
        };

        assert_eq!(run.dist, 0);
        assert_eq!(run.position, 0);
        assert_eq!(run.tokens.len(), 3);
    }

    #[test]
    fn test_run_read_until_true4() {
        let tokens = &[
            make_token(Symbol::Identifier("first"), 1),
            make_token(Symbol::Identifier("second"), 2),
            make_token(Symbol::Identifier("target"), 3),
        ];
        let run = Run::new(tokens);
        let deny = Deny::new().insert(Symbol::Identifier("target"));

        assert_eq!(run.dist, 0);
        assert_eq!(run.position, 0);
        assert_eq!(run.tokens.len(), 3);

        let (token, span) = match run.read_until_true(|s| s == &Symbol::Identifier("target"), deny)
        {
            ReadResult::Some(token, span) => (token, span),
            ReadResult::None(_) => panic!("Expected Some, got None"),
        };

        assert_eq!(token.symbol, Symbol::Identifier("target"));
        assert_eq!(token.index_from, 2);
        assert_eq!(token.index_to, 3);
        assert_eq!(span.range.index_from, 0);
        assert_eq!(span.range.index_to, 3);
    }

    #[test]
    fn test_run_until_some1() {
        let tokens = &[
            make_token(Symbol::Identifier("first"), 1),
            make_token(Symbol::Identifier("second"), 2),
        ];
        let run = Run::new(tokens);
        let deny = Deny::new();
        let pred = |s: &Symbol| match &s {
            Symbol::Identifier(name) if *name == "second" => Some(42),
            _ => None,
        };

        let (value, span) = match run.read_until_some(pred, deny) {
            ReadResult::Some(value, span) => (value, span),
            ReadResult::None(_) => panic!("Expected Some, got None"),
        };

        assert_eq!(value, 42);
        assert_eq!(span.last().symbol, Symbol::Identifier("second"));
        assert_eq!(span.dist, 1);
        assert_eq!(span.range.index_from, 0);
        assert_eq!(span.range.index_to, 2);
        assert_eq!(span.init().len(), 1);

        let first_init_token = span.init().get(0).unwrap();

        assert_eq!(first_init_token.symbol, Symbol::Identifier("first"));
        assert_eq!(first_init_token.index_from, 0);
        assert_eq!(first_init_token.index_to, 1);

        let run = span.cont();

        assert_eq!(run.remaining_input_len(), 0);
    }
}
