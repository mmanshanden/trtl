use super::lex::Token;


pub type Tokens<'a> = &'a [Token<'a>];


#[derive(Debug, Clone)]
pub enum Marker<'a> {
    Number(&'a str),
    Identifier(&'a str),
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

pub type Echo<'a> = Vec<Marker<'a>>;

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

    pub fn dist(&self) -> usize {
        self.dist
    }

    pub fn remaining_input_len(&self) -> usize {
        self.input.len()
    }

    pub fn next(&self, deny: impl Contains<'a>) -> Option<(usize, Tokens<'a>, Token<'a>, Run<'a>)> {
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

    pub fn find(
        &self,
        pred: impl Fn(&'a Token<'a>) -> bool,
        deny: impl Contains<'a>
    ) -> Option<(usize, Tokens<'a>, Token<'a>, Run<'a>)> {
        let mut dist = 0;

        for (idx, token) in self.input.iter().enumerate() {
            if pred(token) {
                let next = Run {
                    dist: self.dist + dist,
                    input: &self.input[idx..],
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

    pub fn first_where(
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

    pub fn first_where_some<T>(
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
