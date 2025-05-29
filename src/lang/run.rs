use super::lex::Token;


pub type Tokens<'a> = &'a [Token<'a>];


#[derive(Debug, Clone)]
pub enum Marker<'a> {
    Number(&'a str),
    Identifier(&'a str),
    Function(&'a str),
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

pub trait Contains<'a> {
    fn contains(&self, token: &'a Token<'a>) -> bool;
}

impl<'a, F> Contains<'a> for F
where
    F: Fn(&'a Token<'a>) -> bool,
{
    fn contains(&self, token: &'a Token<'a>) -> bool {
        self(token)
    }
}

#[derive(Debug)]
pub struct Span<'a> {
    dist: usize,
    tokens: Tokens<'a>,
    next: Run<'a>
}

impl<'a> Span<'a> {
    pub fn all(&self) -> Tokens<'a> {
        self.tokens
    }

    pub fn tail(&self) -> Tokens<'a> {
        &self.tokens[..self.tokens.len() - 1]
    }

    pub fn head(&self) -> Token<'a> {
        self.tokens[self.tokens.len() - 1]
    }

    pub fn peek(&self) -> &'a Token<'a> {
        self.next.input.get(0).unwrap_or(&Token::Eof)
    }

    pub fn next(self) -> Run<'a> {
        self.next
    }
}

pub enum Pass<'a, T> {
    Some(T, Span<'a>),
    None(Run<'a>)
}

#[derive(Clone, Debug)]
pub struct Run<'a> {
    dist: usize,
    input: Tokens<'a>,
}

impl<'a> Run<'a> {
    pub fn new(input: Tokens<'a>) -> Self {
        Run {
            dist: 0,
            input,
        }
    }

    fn pass<T>(self, dist: usize, value: T, offset: usize) -> Pass<'a, T> {
        Pass::Some(
            value, 
            Span { 
                dist: dist, 
                tokens: &self.input[..offset], 
                next: Run { 
                    dist: self.dist + dist, 
                    input: &self.input[offset..] 
                } 
            }
        )
    }

    pub fn dist(&self) -> usize {
        self.dist
    }

    pub fn remaining_input_len(&self) -> usize {
        self.input.len()
    }

    pub fn next_pass(self, deny: impl Contains<'a>) -> Pass<'a, ()> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if deny.contains(token) {
                return self.pass(dist, (), idx + 1);
            }

            dist += token.dist();
        }

        Pass::None(self)
    }

    pub fn until_where(
        self,
        pred: impl Fn(&'a Token<'a>) -> bool,
        deny: impl Contains<'a>
    ) -> Pass<'a, ()> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if pred(token) {
                return self.pass(dist, (), idx);
            }

            if deny.contains(token) {
                return Pass::None(self);
            }

            dist += token.dist();
        }

        Pass::None(self)
    }


    pub fn first_where(
        self,
        pred: impl Fn(&'a Token<'a>) -> bool,
        deny: impl Contains<'a>,
    ) -> Pass<'a, ()> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if pred(token) {
                return self.pass(dist, (), idx + 1);
            }

            if deny.contains(token) {
                return Pass::None(self);
            }

            dist += token.dist();
        }

        Pass::None(self)
    }

    /// Traverses the input until the given predicate returns a `Some` value. Tokens
    /// cannot be skipped when they are contained in the provided `deny` set.
    ///
    /// The predicate `pred` is provided two arguments: 
    ///   1. `token`, the current token in the stream
    ///   2. `peek`, the token that comes after `token`, or `Token::Eof` if `token` 
    ///              is at the end of the stream.
    /// 
    /// The returned tuple contains in order:
    ///   1. The `dist` distance value of skipped tokens.
    ///   2. The value returned by `pred`.
    ///   3. A `tokens` slice of skipped tokens.
    ///   4. The first `token` for which the predicate returned `true`.
    ///   5. The `next` run struct that can be used for parsing the remaining
    ///      input.
    ///            
    /// A `None` is returned when the predicate never matches any input.
    pub fn first_where_some<T>(
        self,
        pred: impl Fn(&'a Token<'a>) -> Option<T>,
        deny: impl Contains<'a>,
    ) -> Pass<'a, T> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if let Some(value) = pred(token) {
                return self.pass(dist, value, idx + 1);
            }

            if deny.contains(token) {
                return Pass::None(self);
            }

            dist += token.dist();
        }

        Pass::None(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_new() {
        let tokens = &[Token::Identifier("a"), Token::Identifier("b")];
        let run = Run::new(tokens);

        assert_eq!(run.dist(), 0);
        assert_eq!(run.remaining_input_len(), 2);
    }
}
