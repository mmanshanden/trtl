use super::lex::{Token, Tokens};

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
    next: Run<'a>,
}

impl<'a> Span<'a> {
    pub fn all(&self) -> Tokens<'a> {
        self.tokens
    }

    pub fn init(&self) -> Tokens<'a> {
        if self.tokens.len() > 1 {
            &self.tokens[..self.tokens.len() - 1]
        } else {
            &[]
        }
    }

    pub fn last(&self) -> Option<Token<'a>> {
        self.tokens.get(self.tokens.len() - 1).copied()
    }

    pub fn peek(&self) -> Option<&'a Token<'a>> {
        self.next.input.get(0)
    }

    pub fn next(self) -> Run<'a> {
        self.next
    }

    pub fn dissect<Pred, MapTrue, MapFalse, Append, T>(
        &self,
        pred: Pred,
        map_true: MapTrue,
        map_false: MapFalse,
        append: Append,
    ) -> Vec<T>
    where
        Pred: Fn(&'a Token<'a>) -> bool,
        MapTrue: Fn(Token<'a>) -> T,
        MapFalse: Fn(Tokens<'a>) -> T,
        Append: FnOnce(Token<'a>) -> T,
    {
        let mut result = Vec::new();
        let mut i = 0;

        while i < self.init().len() {
            if pred(&self.tokens[i]) {
                result.push(map_true(self.tokens[i]));
                i += 1;
                continue;
            }

            let mut j = i + 1;

            while j < self.init().len() {
                if pred(&self.tokens[j]) {
                    break;
                }

                j += 1;
            }

            result.push(map_false(&self.tokens[i..j]));

            i = j;
        }

        if let Some(marker) = self.last().map(append) {
            result.push(marker);
        }

        result
    }
}

pub enum ReadResult<'a, T> {
    Some(T, Span<'a>),
    None(Run<'a>),
}

#[derive(Clone, Debug)]
pub struct Run<'a> {
    dist: usize,
    input: Tokens<'a>,
}

impl<'a> Run<'a> {
    pub fn new(input: Tokens<'a>) -> Self {
        Run { dist: 0, input }
    }

    fn to_read_result<T>(self, dist: usize, value: T, offset: usize) -> ReadResult<'a, T> {
        ReadResult::Some(
            value,
            Span {
                dist: dist,
                tokens: &self.input[..offset],
                next: Run {
                    dist: self.dist + dist,
                    input: &self.input[offset..],
                },
            },
        )
    }

    pub fn dist(&self) -> usize {
        self.dist
    }

    pub fn remaining_input_len(&self) -> usize {
        self.input.len()
    }

    pub fn next_pass(self, deny: impl Contains<'a>) -> ReadResult<'a, ()> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if deny.contains(token) {
                return self.to_read_result(dist, (), idx + 1);
            }

            dist += token.dist();
        }

        ReadResult::None(self)
    }

    // this is cursed
    pub fn until_where(
        self,
        pred: impl Fn(&'a Token<'a>) -> bool,
        deny: impl Contains<'a>,
    ) -> ReadResult<'a, ()> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if pred(token) {
                return self.to_read_result(dist, (), idx);
            }

            if deny.contains(token) {
                return ReadResult::None(self);
            }

            dist += token.dist();
        }

        ReadResult::None(self)
    }

    pub fn first_where(
        self,
        pred: impl Fn(&'a Token<'a>) -> bool,
        deny: impl Contains<'a>,
    ) -> ReadResult<'a, Token<'a>> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if pred(token) {
                return self.to_read_result(dist, token.clone(), idx + 1);
            }

            if deny.contains(token) {
                return ReadResult::None(self);
            }

            dist += token.dist();
        }

        ReadResult::None(self)
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
    ) -> ReadResult<'a, T> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if let Some(value) = pred(token) {
                return self.to_read_result(dist, value, idx + 1);
            }

            if deny.contains(token) {
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

    #[test]
    fn test_run_new() {
        let tokens = &[Token::Identifier("a"), Token::Identifier("b")];
        let run = Run::new(tokens);

        assert_eq!(run.dist(), 0);
        assert_eq!(run.remaining_input_len(), 2);
    }
}
