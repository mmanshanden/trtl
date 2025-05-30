use std::clone;

use super::{ast::{Entry, Expr, Program, Stmt}, lex::{Token, Tokens}, run::{Contains, Pass, Run}, Context, Marker, Markers};

fn mark_as_whitespace_or_unexpected_token<'a, F>(tokens: Tokens<'a>, token: Token<'a>, mark: F) -> Markers<'a>
where
    F: Fn(Token<'a>) -> Marker<'a>
{
    if tokens.is_empty() {
        return vec![mark(token)];
    }

    let mut output = Vec::new();

    for (i, t) in tokens.iter().enumerate() {
        let tag = match t {
            Token::Whitespace(str) => Some(Marker::Whitespace(str)),
            Token::Comment(str) => Some(Marker::Comment(str)),
            Token::LineBreak => Some(Marker::LineBreak),
            _ => None
        };

        if tag.is_none() {
            continue;
        }

        if i > 0 {
            output.push(Marker::UnexpectedToken {
                expected: token,
                actual: &tokens[..i],
            });
        }

        output.push(tag.unwrap());

        return [
            output, 
            mark_as_whitespace_or_unexpected_token(&tokens[i + 1..], token, mark)
        ].concat();
    }

    output.push(Marker::UnexpectedToken {
        expected: token,
        actual: tokens,
    });

    output.push(mark(token));

    output
}

pub type Deny<'a> = Vec<Token<'a>>;

impl<'a> Contains<'a> for Deny<'a> {
    fn contains(&self, token: &'a Token<'a>) -> bool {
        self.iter().any(|t| t == token)
    }
}



#[derive(Debug, Clone)]
pub enum ParseResult<'a, T> {
    Success(T, Markers<'a>, Run<'a>),
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

    pub fn unwrap(self) -> (T, Markers<'a>, Run<'a>) {
        match self {
            Self::Success(value, fragments, run) => (value, fragments, run),
            Self::Error => panic!("Called `unwrap` on an `Error` value"),
        }
    }

    pub fn dist(&self) -> usize {
        match self {
            Self::Success(_, _, run) => run.dist(),
            Self::Error => panic!("Called `dist` on an `Error` value"),
        }
    }

    pub fn remaining_input_len(&self) -> usize {
        match self {
            Self::Success(_, _, run) => run.remaining_input_len(),
            Self::Error => panic!("Called `remaining_input_len` on an `Error` value"),
        }
    }

    pub fn map<U>(self, map: impl Fn(T) -> U) -> ParseResult<'a, U> {
        match self {
            Self::Success(value, fragments, run) => ParseResult::Success(map(value), fragments, run),
            Self::Error => ParseResult::Error,
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
            let (value, markers, run) = match self.parse(run, deny) {
                ParseResult::Success(value, markers, run) => (value, markers, run),
                ParseResult::Error => return ParseResult::Error,
            };

            ParseResult::Success(map(value), markers, run)
        }
    }

    fn optional(self) -> impl Parser<'a, Option<T>> 
    where
        Self: Sized 
    {
        move |run: Run<'a>, deny: Deny<'a>| {
            match self.parse(run.clone(), deny) {
                ParseResult::Success(value, markers, run) => ParseResult::Success(Some(value), markers, run),
                ParseResult::Error => ParseResult::Success(None, vec![], run)
            }
        }
    }

    fn deny(self, deny: Token<'a>) -> impl Parser<'a, T> 
    where
        Self: Sized 
    {
        move |run: Run<'a>, parent: Deny<'a>| {
            let deny = vec![deny];
            let deny = [deny, parent].concat();
            self.parse(run, deny)
        }
    }

    fn update_context(self, map: impl Fn(T, Context) -> Context) -> impl Parser<'a, T>
    where
        T: Clone,
        Self: Sized
    {
        move |run: Run<'a>, deny: Deny<'a>| {
            match self.parse(run, deny) {
                ParseResult::Error => ParseResult::Error,
                ParseResult::Success(val, markers, run) => {
                    let run = run.update_context(|ctx: Context| {
                        map(val.clone(), ctx)
                    });

                    ParseResult::Success(val, markers, run)
                }
            }
        }
    }

    fn check_context(self, pred: impl Fn(T, &Context) -> bool) -> impl Parser<'a, T>
    where
        T: Clone,
        Self: Sized
    {
        move |run: Run<'a>, deny: Deny<'a>| {
            match self.parse(run, deny) {
                ParseResult::Error => ParseResult::Error,
                ParseResult::Success(val, markers, run) => {
                    if pred(val.clone(), &run.context) {
                        return ParseResult::Success(val, markers, run)
                    }

                    return ParseResult::Error;
                }
            }
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
                let (value, markers, run) = result.unwrap();
                println!("\t[{}] [OK] {:?}", name, run);
                ParseResult::Success(value, markers, run)
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


fn expect<'a, U>(expect: Token<'static>, mark: U) -> impl Parser<'a, ()>
where
    U: Fn(Token<'a>) -> Marker<'a>,
{
    let parser = expect_pred(move |&t| t == expect, mark);

    move |run: Run<'a>, deny: Deny<'a>| {
        match parser.parse(run, deny) {
            ParseResult::Success(_, markers, run) => ParseResult::Success((), markers, run),
            ParseResult::Error => ParseResult::Error,
        }
    }
}


fn expect_pred<'a, U>(pred: impl Fn(&'a Token<'a>) -> bool, mark: U) -> impl Parser<'a, Token<'a>>
where
    U: Fn(Token<'a>) -> Marker<'a>,
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let span = match run.first_where(&pred, deny) {
            Pass::None(_) => return ParseResult::Error,
            Pass::Some(_, span) => span
        };

        let markers = mark_as_whitespace_or_unexpected_token(span.tail(), span.head(), &mark);

        ParseResult::Success(
            span.head(), 
            markers, 
            span.next()
        )
    }
}


fn expect_next<'a, U>(expect: Token<'static>, mark: U) -> impl Parser<'a, ()>
where
    U: Fn(Token<'a>) -> Marker<'a>
{
    move |run: Run<'a>, _: Deny<'a>| {
        let span = match run.next_pass(|t: &Token| t.dist() > 0) {
            Pass::None(_) => return ParseResult::Error,
            Pass::Some(_, span) => span,
        };

        if span.head() != expect {
            return ParseResult::Error;
        }

        let markers = mark_as_whitespace_or_unexpected_token(
            span.tail(), 
            span.head(), 
            &mark
        );

        ParseResult::Success((), markers, span.next())
    }
}

fn expect_some<'a, F, U, T>(expect: F, mark: U) -> impl Parser<'a, T>
where
    T: Clone,
    F: Fn(&'a Token<'a>) -> Option<T>,
    U: Fn(T, Token<'a>) -> Marker<'a>,
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let (val, span) = match run.first_where_some(&expect, deny) {
            Pass::None(_) => return ParseResult::Error,
            Pass::Some(val, span) => (val, span)
        };

        let markers = mark_as_whitespace_or_unexpected_token(
            span.tail(), 
            span.head(), 
            |token| mark(val.clone(), token)
        );

        ParseResult::Success(val, markers, span.next())
    }
}

fn choice<'a, P, T>(parsers: Vec<P>) -> impl Parser<'a, T> 
where 
    P: Parser<'a, T> 
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let mut best = ParseResult::Error;

        for parser in &parsers {
            let inner_run = run.clone();
            let inner_deny = deny.clone();
            let result = parser.parse(inner_run, inner_deny);

            if result.is_err() {
                continue;
            }

            let (value, markers, next) = result.unwrap();

            if next.dist() == run.dist() {
                best = ParseResult::Success(value, markers, next);
                break;
            }

            if best.is_err() { 
                best = ParseResult::Success(value, markers, next);
                continue;
            }

            if next.dist() < best.dist() {
                best = ParseResult::Success(value, markers, next);
                continue;
            }

            if next.remaining_input_len() < best.remaining_input_len() {
                best = ParseResult::Success(value, markers, next);
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
        let (value1, markers1, run) = match first.parse(run.clone(), deny.clone()) {
            ParseResult::Success(value, markers, run) => (value, markers, run),
            ParseResult::Error => return ParseResult::Error,
        };

        let (value2, markers2, run) = match second.parse(run.clone(), deny.clone()) {
            ParseResult::Success(value, markers, run) => (value, markers, run),
            ParseResult::Error => return ParseResult::Error,
        };

        let markers3 = [markers1, markers2].concat();
        let result3 = combine(value1, value2);

        ParseResult::Success(result3, markers3, run)
    }
}

fn first<'a, A, B, P, Q>(first: P, second: Q) -> impl Parser<'a, A> 
where
    P: Parser<'a, A>,
    Q: Parser<'a, B>,
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let (value1, markers1, run) = match first.parse(run.clone(), deny.clone()) {
            ParseResult::Success(value, markers, run) => (value, markers, run),
            ParseResult::Error => return ParseResult::Error,
        };

        let (_, markers2, run) = match second.parse(run.clone(), deny.clone()) {
            ParseResult::Success(value, markers, run) => (value, markers, run),
            ParseResult::Error => return ParseResult::Error,
        };

        let markers3 = [markers1, markers2].concat();

        ParseResult::Success(value1, markers3, run)
    }
}


fn second<'a, A, B, P, Q>(first: P, second: Q) -> impl Parser<'a, B> 
where
    P: Parser<'a, A>,
    Q: Parser<'a, B>,
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let (_, markers1, run) = match first.parse(run.clone(), deny.clone()) {
            ParseResult::Success(value, markers, run) => (value, markers, run),
            ParseResult::Error => return ParseResult::Error,
        };

        let (value2, markers2, run) = match second.parse(run.clone(), deny.clone()) {
            ParseResult::Success(value, markers, run) => (value, markers, run),
            ParseResult::Error => return ParseResult::Error,
        };

        let markers3 = [markers1, markers2].concat();

        ParseResult::Success(value2, markers3, run)
    }
}

fn many<'a, P, T>(parser: P) -> impl Parser<'a, Vec<T>> 
where
    P: Parser<'a, T>,
{
    move |run: Run<'a>, deny: Deny<'a>| {
        let mut run = run;
        let mut markers = Vec::new();
        let mut values = Vec::new();

        loop {
            let inner_run = run.clone();
            let inner_deny = deny.clone();

            let (value, ts, next) = match  parser.parse(inner_run, inner_deny) {
                ParseResult::Success(value, markers, run) => (value, markers, run),
                ParseResult::Error => break,
            };

            run = next;
            markers = [markers, ts].concat();
            values.push(value);
        }

        ParseResult::Success(values, markers, run)
    }
}

fn wrapped<'a, T>(before: Token<'static>, after: Token<'static>, body: impl Parser<'a, T>) -> impl Parser<'a, T> {
    second(
        expect(before, Marker::Plain),
        first(
            body.deny(after),
            expect(after, Marker::Plain),
        ),
    )
}

fn wrapped_attached<'a, T>(before: Token<'static>, after: Token<'static>, body: impl Parser<'a, T>) -> impl Parser<'a, T> {
    second(
        expect_next(before, Marker::Plain),
        first(
            body.deny(after),
            expect(after, Marker::Plain),
        ),
    )
}


fn seperated_by<'a, T>(parser: impl Parser<'a, T>, separator: Token<'static>) -> impl Parser<'a, Vec<T>> {
    let parser = parser.deny(separator);

    move |run: Run<'a>, deny: Deny<'a>| {

        // parse the first element
        let first_element = parser.parse(run.clone(), deny.clone());

        let (mut values, mut markers, mut run) = match first_element {
            ParseResult::Error => {
                return ParseResult::Success(Vec::new(), Vec::new(), run)
            },
            ParseResult::Success(value, markers, run) => {
                (vec![value], markers, run)
            }
        };

        loop {
            // parse the separator
            match expect(separator, Marker::Plain).parse(run.clone(), deny.clone()) {
                ParseResult::Error => { 
                    break;
                },
                ParseResult::Success(_, mut ts, next) => {
                    markers.append(&mut ts);
                    run = next;
                }
            };

            // parse the next element
            match parser.parse(run.clone(), deny.clone()) {
                ParseResult::Error => { 
                    break;
                },
                ParseResult::Success(value, mut ts, next) => {
                    values.push(value);
                    markers.append(&mut ts);
                    run = next;
                }
            }
        }

        ParseResult::Success(values, markers, run)
    }
}

pub fn parse_expr_primary<'a>() -> impl Parser<'a, Expr> {
    let number_parser = |run: Run<'a>, deny: Deny<'a>| {
        expect_some(Token::number, |number, _| Marker::Number(number)).map(|number| {
            let number = number.parse().unwrap();
            Expr::Num(number)
        }).parse(run, deny)
    };

    let func_call_parser = |run: Run<'a>, deny: Deny<'a>| {
        combine(
            expect_some(Token::identifier, |id, _| Marker::Call(id)),
            wrapped_attached(
                Token::LeftParen,
                Token::RightParen,
                seperated_by(parse_expr(), Token::Comma)
            ),
            |id, args| (id, args)
        ).check_context(|(id, args), ctx| {
            ctx.functions.contains(&(id.to_string(), args.len()))
        }).map(|(id, args)| {
            let id = id.to_string();
            Expr::Call(id, args)
        }).parse(run, deny)
    };

    let identifier_parser = |run: Run<'a>, deny: Deny<'a>| {
        expect_some(
            Token::identifier,
            |id, _| Marker::Identifier(id)
        ).check_context(|id, ctx| {
            ctx.variables.contains_key(id)
        }).map(|id| {
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

#[derive(PartialEq, Clone, Debug)]
enum Operation {
    Equals,
    NotEqual,
    LessThan,
    LessEqualThan,
    GreaterThan,
    GreaterEqualThan,
    Plus,
    Minus,
    Multiply,
    Divide,
}


/// The recurrent case for a given minimum precedence level
/// 
/// See: https://eli.thegreenplace.net/2012/08/02/parsing-expressions-by-precedence-climbing
///
fn parse_expr_1<'a>(min_prec: u8) -> impl Parser<'a, Expr>{
    move |run: Run<'a>, deny: Deny<'a>| {
        let (mut lhs, mut markers, mut run) = match parse_expr_primary().parse(run, deny.clone()) {
            ParseResult::Success(expr, fs, next) => (expr, fs, next),
            ParseResult::Error => return ParseResult::Error,
        };

        let mark = |_, token| Marker::Plain(token);

        let some_operation = |token: &Token<'a>| match token {
            Token::Equals => Some((Operation::Equals, 1, Associativity::Left)),
            Token::NotEqual => Some((Operation::NotEqual, 1, Associativity::Left)),
            Token::LessThan => Some((Operation::LessThan, 1, Associativity::Left)),
            Token::LessEqualThan => Some((Operation::LessEqualThan, 1, Associativity::Left)),
            Token::GreaterThan => Some((Operation::GreaterThan, 1, Associativity::Left)),
            Token::GreaterEqualThan => Some((Operation::GreaterEqualThan, 1, Associativity::Left)),
            Token::Plus => Some((Operation::Plus, 2, Associativity::Left)),
            Token::Minus => Some((Operation::Minus, 2, Associativity::Left)),
            Token::Multiply => Some((Operation::Multiply, 3, Associativity::Left)),
            Token::Divide => Some((Operation::Divide, 3, Associativity::Left)),
            _ => None,
        };

        loop {
            let result = expect_some(some_operation, mark).parse(run.clone(), deny.clone());

            if result.is_err() {
                break;
            }

            let (operation, mut inner_marks, next) = result.unwrap();
            let (operation, prec, assoc) = operation;

            if prec < min_prec {
                break;
            }

            markers.append(&mut inner_marks);
            run = next;

            let new_min_prec = if assoc == Associativity::Left {
                prec + 1
            } else {
                prec
            };

            let rhs = parse_expr_1(new_min_prec).check_context(|expr, ctx| {
                if let Expr::Var(var) = expr {
                    ctx.variables.contains_key(&var)
                } else {
                    true 
                }
            });

            let rhs = match rhs.parse(run.clone(), deny.clone()) {
                ParseResult::Error => break,
                ParseResult::Success(rhs, mut fs, next) => {
                    markers.append(&mut fs);
                    run = next;
                    rhs
                }
            };

            let lhs_boxxed = Box::new(lhs);
            let rhs_boxxed = Box::new(rhs);

            lhs = match operation {
                Operation::Equals => Expr::Equals(lhs_boxxed, rhs_boxxed),
                Operation::NotEqual => Expr::NotEqual(lhs_boxxed, rhs_boxxed),
                Operation::GreaterThan => Expr::GreaterThan(lhs_boxxed, rhs_boxxed),
                Operation::GreaterEqualThan => Expr::GreaterEqualThan(lhs_boxxed, rhs_boxxed),
                Operation::LessThan => Expr::LessThan(lhs_boxxed, rhs_boxxed),
                Operation::LessEqualThan => Expr::LessEqualThan(lhs_boxxed, rhs_boxxed),
                Operation::Plus => Expr::Add(lhs_boxxed, rhs_boxxed),
                Operation::Minus => Expr::Sub(lhs_boxxed, rhs_boxxed),
                Operation::Multiply => Expr::Mul(lhs_boxxed, rhs_boxxed),
                Operation::Divide => Expr::Div(lhs_boxxed, rhs_boxxed),
            };
        }

        ParseResult::Success(lhs, markers, run)
    }
}

pub fn parse_expr<'a>() -> impl Parser<'a, Expr> {
    let assignment_parser = |run: Run<'a>, deny: Deny<'a>| {
        combine(
            expect_some(
                Token::identifier,
                |var, _| Marker::Identifier(var)
            ),
            second(
                expect(Token::Assign, Marker::Plain),
                parse_expr(),
            ),
            |var, expr| (var.to_string(), expr)
        ).update_context(|(var, _), ctx| {
            let mut variables = ctx.variables;
            variables.insert(var.clone(), ctx.scope);

            Context { variables: variables, functions: ctx.functions, scope: ctx.scope }
        }).map(|(var, expr)| {
            Expr::Assign(var, Box::new(expr))
        }).parse(run, deny)
    };
    
    let any_expr_parser = |run: Run<'a>, deny: Deny<'a>| {
        parse_expr_1(0).parse(run, deny)
    };

    choice(vec![
        assignment_parser,
        any_expr_parser
    ])
}

fn parse_stmt_block<'a>() -> impl Parser<'a, Stmt> {
    // implements scope drop
    |run: Run<'a>, deny: Deny<'a>| {
        let origin = run.context.clone();

        let parser = second(
            expect(Token::LeftBrace, Marker::Plain).update_context(|_, ctx| {
                Context { 
                    variables: ctx.variables, 
                    functions: ctx.functions, 
                    scope: super::Scope::Local 
                }
            }),
            first(
                many(parse_stmt()).deny(Token::RightBrace).map(Stmt::Scope),
                expect(Token::RightBrace, Marker::Plain),
            )        
        );
        
        match parser.parse(run, deny) {
            ParseResult::Error => ParseResult::Error,
            ParseResult::Success(result, markers, next) => {
                ParseResult::Success(result, markers, next.update_context(|_| origin.clone()))
            }
        }
    }
}

fn parse_stmt_expr<'a>() -> impl Parser<'a, Stmt> {
    first(
        parse_expr().deny(Token::SemiColon).map(Stmt::Expr),
        expect(Token::SemiColon, Marker::Plain)
    )
}

pub fn parse_stmt_move<'a>() -> impl Parser<'a, Stmt> {
    combine(
        expect_pred(|&t| matches!(t, Token::Forward | Token::Left | Token::Right), Marker::Move),
        first(
            parse_expr().deny(Token::SemiColon),
            expect(Token::SemiColon, Marker::Plain),
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
    let parse_if = second(
        expect(Token::If, Marker::Keyword),
        combine(
            parse_expr().deny(Token::LeftBrace),
            parse_stmt_block().deny(Token::Else),
            |expr, stmt| (expr, stmt)
        ),
    );

    let parse_else = |run: Run<'a>, deny: Deny<'a>| {
        let is_else = |&token| token == Token::Else;
        let deny_any = |t: &Token| t.dist() > 0;

        let span = match run.first_where(is_else, deny_any) {
            Pass::None(run) => return ParseResult::Success(None, Vec::new(), run),
            Pass::Some(_, span) => span,
        };

        let markers = mark_as_whitespace_or_unexpected_token(span.tail(), Token::Else, Marker::Keyword);

        match parse_stmt_block().parse(span.next(), deny) {
            ParseResult::Error => ParseResult::Error,
            ParseResult::Success(stmt, stmt_markers, next) => {
                ParseResult::Success(Some(stmt), [markers, stmt_markers].concat(), next)
            }
        }
    };

    combine(
        parse_if, 
        parse_else, 
        |(if_expr, if_body), else_block| {
            if let Some(else_body) = else_block {
                Stmt::IfElse(if_expr, Box::new(if_body), Box::new(else_body))
            } else {
                Stmt::If(if_expr, Box::new(if_body))
            }
        }
    )
}

fn parse_stmt_while<'a>() -> impl Parser<'a, Stmt> {
    second(
        expect(Token::While, Marker::Keyword),
        combine(
            parse_expr().deny(Token::LeftBrace),
            parse_stmt_block().map(Box::new),
            Stmt::While
        ),
    )
}

fn parse_stmt_return<'a>() -> impl Parser<'a, Stmt> {
    first(
        combine(
            expect(Token::Return, Marker::Flow),
            parse_expr().optional(),
            |_, expr| Stmt::Return(expr)
        ),
        expect(Token::SemiColon, Marker::Plain)
    )
}

fn parse_stmt<'a>() -> impl Parser<'a, Stmt> {
    let is_stmt_token = |token: &Token<'a>| match token {
        Token::Number(_) => true,
        Token::Identifier(_) => true,
        Token::LeftParen => true,
        Token::LeftBrace => true,
        Token::If => true,
        Token::While => true,
        Token::Forward => true,
        Token::Left => true,
        Token::Right => true,
        Token::Return => true,
        _ => false,
    };

    move |run: Run<'a>, deny: Deny<'a>| {
        let span = match run.until_where(is_stmt_token, deny.clone()) {
            Pass::None(_) => return ParseResult::Error,
            Pass::Some(_, span) => span
        };

        let mut markers1 = mark_as_whitespace_or_unexpected_token(span.all(), *span.peek(), Marker::Plain);
        markers1.pop();

        let parse_result = match span.peek() {
            Token::Number(_) => parse_stmt_expr().parse(span.next(), deny),
            Token::Identifier(_) => parse_stmt_expr().parse(span.next(), deny),
            Token::LeftParen => parse_stmt_expr().parse(span.next(), deny),
            Token::LeftBrace => parse_stmt_block().parse(span.next(), deny),
            Token::If => parse_stmt_if().parse(span.next(), deny),
            Token::While => parse_stmt_while().parse(span.next(), deny),
            Token::Forward => parse_stmt_move().parse(span.next(), deny),
            Token::Left => parse_stmt_move().parse(span.next(), deny),
            Token::Right => parse_stmt_move().parse(span.next(), deny),
            Token::Return => parse_stmt_return().parse(span.next(), deny),
            _ => unreachable!()
        };

        if parse_result.is_err() {
            return ParseResult::Error;
        }

        let (value, markers2, next) = parse_result.unwrap();

        return ParseResult::Success(value, vec![markers1, markers2].concat(), next);
    }
}

pub fn parse_func<'a>() -> impl Parser<'a, Entry> {
    let params_parser = wrapped_attached(
        Token::LeftParen, 
        Token::RightParen,
        seperated_by(
            expect_some(Token::identifier, |id, _| Marker::Identifier(id)),
            Token::Comma
        )
    );

    let header_parser = combine(
        expect(Token::Func, Marker::Keyword),
        combine(
            expect_some(
                Token::identifier,
                |id, _| Marker::Function(id)
            ),
            params_parser,
            |name, params| (name, params)
        ),
        |_, header| header
    ).update_context(|(func, vars), ctx| {
        let mut variables = ctx.variables;
        let mut functions = ctx.functions;
        
        functions.insert((func.to_string(), vars.len()));

        for var in vars {
            variables.insert(var.to_string(), super::Scope::Local);
        }

        Context {
            variables,
            functions,
            scope: ctx.scope,
        }
    });

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

    first(
        many(
            choice(vec![
                parse_stmt,
                parse_func,
            ]),
        ),
        expect(Token::Eof, Marker::Plain),
    )
}
