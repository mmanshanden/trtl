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

#[derive(Debug)]
pub struct Reading<'a> {
    dist: usize,
    source: Run<'a>,
    index_from: usize,
    index_to: usize,
}

/// A reading represents a contiguous sequence of one or more tokens read from the
/// input stream by a Run instance, ending at the token that satisfied the read
/// condition.
///
impl<'a> Reading<'a> {
    pub fn all(&self) -> Tokens<'a> {
        &self.source.tokens[..self.index_to]
    }

    pub fn init(&self) -> Tokens<'a> {
        &self.source.tokens[..self.index_to - 1]
    }

    pub fn last(&self) -> Token<'a> {
        self.source.tokens[self.index_to - 1]
    }

    pub fn peek_symbol(&self) -> Option<&'a Symbol<'a>> {
        self.source
            .tokens
            .get(self.index_to)
            .map(|token| &token.symbol)
    }

    pub fn revert(self) -> Run<'a> {
        self.source
    }

    pub fn resume(self) -> Run<'a> {
        Run {
            dist: self.source.dist + self.dist,
            tokens: &self.source.tokens,
            position: self.index_to,
        }
    }

    pub fn index_from(&self) -> usize {
        self.index_from
    }

    pub fn index_to(&self) -> usize {
        self.index_to
    }
}

pub enum ReadResult<'a, T> {
    Some(T, Reading<'a>),
    None(Run<'a>),
}

impl<'a, T> ReadResult<'a, T> {
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

    pub fn position(&self) -> usize {
        self.position
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
    pub fn consume(self) -> ReadResult<'a, Token<'a>> {
        if let Some(token) = self.stream().first() {
            ReadResult::Some(
                *token,
                Reading {
                    dist: 0,
                    index_from: self.position,
                    index_to: self.position + 1,
                    source: self,
                },
            )
        } else {
            ReadResult::None(self)
        }
    }

    pub fn consume_if_true(self, pred: impl Fn(&Symbol<'a>) -> bool) -> ReadResult<'a, Token<'a>> {
        if let Some(token) = self.stream().first()
            && pred(&token.symbol)
        {
            ReadResult::Some(
                *token,
                Reading {
                    dist: 0,
                    index_from: self.position,
                    index_to: self.position + 1,
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
                    Reading {
                        dist,
                        index_from: self.position,
                        index_to: self.position + idx + 1,
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

    // TODO: rename to read_where_some
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
                    Reading {
                        dist: dist,
                        index_from: self.position,
                        index_to: self.position + idx + 1,
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
    use crate::lang::Lexer;

    use super::*;

    fn create_tokens<'a>(input: &'a str) -> Vec<Token<'a>> {
        let mut lexer = Lexer::new(input);
        lexer.collect()
    }

    #[test]
    fn test_run_new() {
        let tokens = create_tokens("");
        let run = Run::new(&tokens);

        assert_eq!(run.dist, 0);
        assert_eq!(run.position, 0);
        assert_eq!(run.remaining_input_len(), 1);

        assert_eq!(run.tokens.get(0).unwrap().symbol, Symbol::Eof);
    }

    #[test]
    fn test_run_consume_next1() {
        let tokens = &[];
        let run = Run::new(tokens);

        match run.consume() {
            ReadResult::Some(_, _) => panic!("Expected None, got Some"),
            ReadResult::None(_) => (),
        }
    }

    #[test]
    fn test_run_consume_next2() {
        let tokens = create_tokens("first second");
        let run = Run::new(&tokens);

        match run.consume() {
            ReadResult::Some(_, reading) => {
                assert_eq!(reading.dist, 0);
                assert_eq!(reading.index_from, 0);
                assert_eq!(reading.index_to, 1);
                assert_eq!(reading.init().len(), 0);
                assert_eq!(reading.last().symbol, Symbol::Identifier("first"));
                assert_eq!(reading.last().index_from, 0);
                assert_eq!(reading.last().index_to, 5);
            }
            ReadResult::None(_) => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_run_read_until_true1() {
        let tokens = create_tokens("target other");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        let (token, reading) =
            match run.read_until_true(|&s| s == Symbol::Identifier("target"), deny) {
                ReadResult::Some(token, reading) => (token, reading),
                ReadResult::None(_) => panic!("Expected Some, got None"),
            };

        assert_eq!(token.symbol, Symbol::Identifier("target"));
        assert_eq!(reading.last(), token);
        assert_eq!(reading.dist, 0);
        assert_eq!(reading.index_from, 0);
        assert_eq!(reading.index_to, 1);
        assert_eq!(reading.init().len(), 0);

        let (token, reading) = match reading.resume().consume() {
            ReadResult::Some(token, reading) => (token, reading),
            ReadResult::None(_) => panic!("Expected Some, got None"),
        };

        assert_eq!(reading.dist, 0);
        assert_eq!(reading.index_from, 1);
        assert_eq!(reading.index_to, 2);
        assert_eq!(token.symbol, Symbol::Whitespace(" "));
        assert_eq!(token.index_from, 6);
        assert_eq!(token.index_to, 7);

        let (token, reading) = match reading.resume().consume() {
            ReadResult::Some(token, reading) => (token, reading),
            ReadResult::None(_) => panic!("Expected Some, got None"),
        };

        assert_eq!(reading.dist, 0);
        assert_eq!(reading.index_from, 2);
        assert_eq!(reading.index_to, 3);
        assert_eq!(token.symbol, Symbol::Identifier("other"));
        assert_eq!(token.index_from, 7);
        assert_eq!(token.index_to, 12);
    }

    #[test]
    fn test_run_read_until_true2() {
        let tokens = create_tokens("first target last");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        let (token, reading) =
            match run.read_until_true(|s| s == &Symbol::Identifier("target"), deny) {
                ReadResult::Some(token, reading) => (token, reading),
                ReadResult::None(_) => panic!("Expected Some, got None"),
            };

        assert_eq!(token, reading.last());
        assert_eq!(token.symbol, Symbol::Identifier("target"));

        assert_eq!(reading.dist, 1);
        assert_eq!(reading.index_from, 0);
        assert_eq!(reading.index_to, 3);

        let init = reading.init();
        assert_eq!(init.len(), 2);
        assert_eq!(init.get(0).unwrap().symbol, Symbol::Identifier("first"));
        assert_eq!(init.get(1).unwrap().symbol, Symbol::Whitespace(" "));

        let (token, reading) = match reading.resume().consume() {
            ReadResult::Some(token, reading) => (token, reading),
            ReadResult::None(_) => panic!("Expected Some, got None"),
        };

        assert_eq!(reading.dist, 0);
        assert_eq!(reading.index_from, 3);
        assert_eq!(reading.index_to, 4);
        assert_eq!(token.symbol, Symbol::Whitespace(" "));

        let (token, reading) = match reading.resume().consume() {
            ReadResult::Some(token, reading) => (token, reading),
            ReadResult::None(_) => panic!("Expected Some, got None"),
        };

        assert_eq!(reading.dist, 0);
        assert_eq!(reading.index_from, 4);
        assert_eq!(reading.index_to, 5);
        assert_eq!(token.symbol, Symbol::Identifier("last"));
    }

    #[test]
    fn test_run_read_until_true3() {
        let tokens = create_tokens("first second target");
        let run = Run::new(&tokens);
        let deny = Deny::new().insert(Symbol::Identifier("second"));

        assert_eq!(run.dist, 0);
        assert_eq!(run.position, 0);
        assert_eq!(run.tokens.len(), 5);

        let run = match run.read_until_true(|s| s == &Symbol::Identifier("target"), deny) {
            ReadResult::Some(_, _) => panic!("Expected None, got Some"),
            ReadResult::None(run) => run,
        };

        assert_eq!(run.dist, 0);
        assert_eq!(run.position, 0);
        assert_eq!(run.tokens.len(), 5);
    }

    #[test]
    fn test_run_read_until_true4() {
        let tokens = create_tokens("first second target");
        let run = Run::new(&tokens);
        let deny = Deny::new().insert(Symbol::Identifier("target"));

        assert_eq!(run.dist, 0);
        assert_eq!(run.position, 0);
        assert_eq!(run.tokens.len(), 3);

        let (token, reading) =
            match run.read_until_true(|s| s == &Symbol::Identifier("target"), deny) {
                ReadResult::Some(token, reading) => (token, reading),
                ReadResult::None(_) => panic!("Expected Some, got None"),
            };

        assert_eq!(token.symbol, Symbol::Identifier("target"));
        assert_eq!(token.index_from, 2);
        assert_eq!(token.index_to, 3);
        assert_eq!(reading.index_from, 0);
        assert_eq!(reading.index_to, 3);
    }

    #[test]
    fn test_run_until_some1() {
        let tokens = create_tokens("first second");
        let run = Run::new(&tokens);
        let deny = Deny::new();
        let pred = |s: &Symbol| match &s {
            Symbol::Identifier(name) if *name == "second" => Some(42),
            _ => None,
        };

        let (value, reading) = match run.read_until_some(pred, deny) {
            ReadResult::Some(value, reading) => (value, reading),
            ReadResult::None(_) => panic!("Expected Some, got None"),
        };

        assert_eq!(value, 42);
        assert_eq!(reading.last().symbol, Symbol::Identifier("second"));
        assert_eq!(reading.dist, 1);
        assert_eq!(reading.index_from, 0);
        assert_eq!(reading.index_to, 3);

        let init = reading.init();
        assert_eq!(reading.init().len(), 2);
        assert_eq!(
            reading.init().get(0).unwrap().symbol,
            Symbol::Whitespace(" ")
        );
        assert_eq!(
            reading.init().get(1).unwrap().symbol,
            Symbol::Identifier("first")
        );

        let run = reading.resume();

        assert_eq!(run.remaining_input_len(), 0);
    }
}
