use super::lex::{Loc, Span, Token};


pub type Tokens<'a> = &'a [Span<Token<'a>>];
pub type Stack<'a> = Vec<Token<'a>>;

#[derive(Clone, Debug)]
pub struct Run<'a> {
    pub dist: i32,
    pub ops: Ops<'a>,
    pub tokens: Tokens<'a>,
}

impl<'a> Run<'a> {
    pub fn new(tokens: &'a Vec<Span<Token>>) -> Self {
        Self {
            dist: 0,
            ops: Ops::Nil,
            tokens
        }
    }
}

impl<'a> Run<'a> {
    pub fn token(&self) -> Token<'a> {
        self.tokens[0].value
    }

    // pub fn current_line(&self) -> usize {
    //     self.tokens[0].from().line()
    // }

    /// Advances the run by 1 token.
    ///
    pub fn advance(self) -> Self {
        Self {
            dist: self.dist,
            tokens: &self.tokens[1..],
            ops: self.ops,
        }
    }

    /// Inserts given token into the run.
    ///
    pub fn insert(self, token: Token<'a>) -> Self {
        let oper = Op::Insert(self.tokens[0].from, token);
        Self {
            dist: self.dist + 1,
            tokens: self.tokens,
            ops: self.ops.append_operation(oper),
        }
    }

    /// Deletes `n` tokens from the run.
    ///
    pub fn delete(self, n: usize) -> Self {
        let oper = Op::Remove(self.tokens[0].from, self.tokens[n - 1].to);

        Self {
            dist: self.dist + (n as i32),
            tokens: &self.tokens[(n + 1)..],
            ops: self.ops.append_operation(oper),
        }
    }

    /// Advances or deletes `n` tokens when `n > 0`.
    /// 
    pub fn advance_by(self, n: usize) -> Self {
        if n == 0 {
            self.advance()
        } else {
            self.delete(n)
        }
    }

    /// Returns the index of the first token that matches the predicate.
    ///
    pub fn first_where(&self, pred: impl Fn(Token) -> bool, deny: &Stack<'a>) -> Option<usize> {
        for (i, t) in self.tokens.iter().enumerate() {
            if deny.contains(&t.value) {
                return None;
            }

            if pred(t.value) {
                return Some(i);
            }
        }
        None
    }

    /// Returns the index and the result of the first token for which the predicate
    /// returns a `Some` value.
    ///
    pub fn first_where_some<T>(
        &self,
        pred: impl Fn(Token<'a>) -> Option<T>,
        deny: &Stack<'a>,
    ) -> Option<(usize, T)> {
        for (i, t) in self.tokens.iter().enumerate() {
            if deny.contains(&t.value) {
                return None;
            }

            if let Some(r) = pred(t.value) {
                return Some((i, r));
            }
        }

        None
    }
}

#[derive(Debug)]
pub enum ParseResult<'a, T> {
    Ok(T, Run<'a>),
    Err(Run<'a>),
}

impl<'a, T> ParseResult<'a, T> {
    // fn is_ok(&self) -> bool {
    //     match self {
    //         ParseResult::Ok(_, _) => true,
    //         _ => false,
    //     }
    // }

    // fn is_err(&self) -> bool {
    //     match self {
    //         ParseResult::Err(_) => true,
    //         _ => false,
    //     }
    // }

    // fn unwrap(self) -> (T, Run<'a>) {
    //     match self {
    //         ParseResult::Ok(val, run) => (val, run),
    //         _ => panic!("Called ParseResult::unwrap on an Err value"),
    //     }
    // }
}

/// An operation that has been used to transform the input to
/// a valid parse tree.
#[derive(Clone, Debug)]
pub enum Op<'a> {
    Insert(Loc, Token<'a>),
    Remove(Loc, Loc),
}

#[derive(Clone, Debug)]
pub enum Ops<'a> {
    Nil,
    Cons(Op<'a>, Box<Ops<'a>>),
}

impl<'a> Ops<'a> {
    fn append_operation(self, op: Op<'a>) -> Ops<'a> {
        Self::Cons(op, Box::new(self))
    }

    // pub fn map<R, F: Fn(Op<'a>) -> R>(self, map: F) -> Vec<R> {
    //     let mut result = Vec::new();
    //     let mut ops = self;

    //     while let Ops::Cons(head, tail) = ops {
    //         result.push(map(head));
    //         ops = *tail;
    //     }

    //     result
    // }
}
