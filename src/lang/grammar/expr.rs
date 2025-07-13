use crate::lang::{
    Expr, Marker, ReadResult, Run, Token,
    parse::{Deny, ParseResult, Parser, seperated_by},
    run::Span,
};

fn into_markers<'a, 'b>(
    span: &'b Span<'a>,
    append: impl FnOnce(Token<'a>) -> Marker<'a>,
) -> Vec<Marker<'a>> {
    span.dissect(
        Token::is_whitespace,
        |token_true| match token_true {
            Token::Whitespace(ws) => Marker::Whitespace(ws),
            Token::LineBreak => Marker::LineBreak,
            Token::Comment(comment) => Marker::Comment(comment),
            _ => unreachable!(),
        },
        |tokens_false| Marker::UnexpectedToken {
            expected: span.last().unwrap(),
            actual: tokens_false,
        },
        append,
    )
}

pub fn parse_expr_unit<'a>() -> impl Parser<'a, Expr> {
    let is_expr_token = |token: &'a Token<'a>| match token {
        Token::Number(_) => true,
        Token::Identifier(_) => true,
        Token::LeftParen => true,
        _ => false,
    };

    move |run: Run<'a>, deny: Deny<'a>| {
        let (token, span) = match run.first_where(is_expr_token, deny.clone()) {
            ReadResult::Some(token, span) => (token, span),
            ReadResult::None(_) => return ParseResult::Error,
        };

        match token {
            Token::Number(number_str) => {
                let number = number_str.parse().unwrap();
                let number = Expr::Num(number);
                
                let markers = into_markers(&span, |_| {
                    return Marker::Number(number_str);
                });

                ParseResult::Success(number, markers, span.next())
            }
            Token::LeftParen => {
                let mut markers = into_markers(&span, |token| {
                    return Marker::Plain(token);
                });

                let parse_deny = deny.insert(Token::RightParen);

                let (expr, run) = match parse_expr().parse(span.next(), parse_deny) {
                    ParseResult::Error => return ParseResult::Error,
                    ParseResult::Success(expr, mut output, run) => {
                        markers.append(&mut output);
                        (expr, run)
                    }
                };

                let run = match run.first_where(|token| token == &Token::RightParen, deny) {
                    ReadResult::None(_) => return ParseResult::Error,
                    ReadResult::Some(_, span) => {
                        let mut output = into_markers(&span, Marker::Plain);
                        markers.append(&mut output);
                        span.next()
                    }
                };

                ParseResult::Success(expr, markers, run)
            }
            Token::Identifier(ident) => {
                let mut markers = into_markers(&span, |_| {
                    return Marker::Identifier(ident);
                });

                if let Some(Token::LeftParen) = span.peek() {
                    match seperated_by(parse_expr(), Token::Comma).parse(span.next(), deny) {
                        ParseResult::Error => return ParseResult::Error,
                        ParseResult::Success(args, mut inner_markers, run) => {
                            markers.append(&mut inner_markers);

                            let expr = Expr::Call(ident.to_string(), args);
                            ParseResult::Success(expr, markers, run)
                        }
                    }
                } else {
                    let ident = ident.to_string();
                    let ident = Expr::Var(ident);

                    ParseResult::Success(ident, markers, span.next())
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Associativity {
    Left,
    Right,
}

#[derive(PartialEq, Clone, Debug)]
enum Operation {
    Assign,
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
fn parse_expr_chain<'a>(min_prec: u8) -> impl Parser<'a, Expr> {
    move |run: Run<'a>, deny: Deny<'a>| {
        let some_operation = |token: &Token<'a>, lhs: &Expr| match token {
            Token::Assign if lhs.var().is_some() => Some((Operation::Assign, 1, Associativity::Right)),
            Token::Equals => Some((Operation::Equals, 2, Associativity::Left)),
            Token::NotEqual => Some((Operation::NotEqual, 2, Associativity::Left)),
            Token::LessThan => Some((Operation::LessThan, 2, Associativity::Left)),
            Token::LessEqualThan => Some((Operation::LessEqualThan, 2, Associativity::Left)),
            Token::GreaterThan => Some((Operation::GreaterThan, 2, Associativity::Left)),
            Token::GreaterEqualThan => Some((Operation::GreaterEqualThan, 2, Associativity::Left)),
            Token::Plus => Some((Operation::Plus, 3, Associativity::Left)),
            Token::Minus => Some((Operation::Minus, 3, Associativity::Left)),
            Token::Multiply => Some((Operation::Multiply, 4, Associativity::Left)),
            Token::Divide => Some((Operation::Divide, 4, Associativity::Left)),
            _ => None,
        };

        let (mut lhs, mut markers, mut run) = match parse_expr_unit().parse(run, deny.clone()) {
            ParseResult::Success(expr, fs, next) => (expr, fs, next),
            ParseResult::Error => return ParseResult::Error,
        };

        loop {
            let (result, span) =
                match run.first_where_some(|token| some_operation(token, &lhs), deny.clone()) {
                    ReadResult::Some(result, span) => (result, span),
                    ReadResult::None(next) => {
                        run = next;
                        break;
                    }
                };

            let (operation, prec, assoc) = result;

            if prec < min_prec {
                run = span.next();
                break;
            }

            markers.append(&mut into_markers(&span, Marker::Plain));

            run = span.next();

            let new_min_prec = if assoc == Associativity::Left {
                prec + 1
            } else {
                prec
            };

            let rhs = match parse_expr_chain(new_min_prec).parse(run.clone(), deny.clone()) {
                ParseResult::Error => break,
                ParseResult::Success(rhs, mut fs, next) => {
                    markers.append(&mut fs);
                    run = next;
                    rhs
                }
            };

            let var = lhs.var().cloned();
            let lhs_boxxed = Box::new(lhs);
            let rhs_boxxed = Box::new(rhs);

            lhs = match operation {
                Operation::Assign => Expr::Assign(var.unwrap(), rhs_boxxed),
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
    parse_expr_chain(0)
}
