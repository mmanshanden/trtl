use crate::lang::{
    Expr, Operator, ReadResult, Run, Symbol,
    parse::{Deny, ParseResult},
    run::Range,
};

#[derive(PartialEq, Clone, Copy, Debug)]
enum Associativity {
    Left,
    Right,
}

fn is_atom(token: &Symbol<'_>) -> bool {
    match token {
        Symbol::Number(_) => true,
        Symbol::Identifier(_) => true,
        Symbol::LeftParen => true,
        _ => false,
    }
}

fn is_some_operation(token: &Symbol<'_>) -> Option<(Operator, u8, Associativity)> {
    match token {
        Symbol::Assign => Some((Operator::Assign, 1, Associativity::Right)),
        Symbol::Equals => Some((Operator::Equal, 2, Associativity::Left)),
        Symbol::NotEqual => Some((Operator::NotEqual, 2, Associativity::Left)),
        Symbol::LessThan => Some((Operator::LessThan, 2, Associativity::Left)),
        Symbol::LessEqualThan => Some((Operator::LessEqualThan, 2, Associativity::Left)),
        Symbol::GreaterThan => Some((Operator::GreaterThan, 2, Associativity::Left)),
        Symbol::GreaterEqualThan => Some((Operator::GreaterEqualThan, 2, Associativity::Left)),
        Symbol::Plus => Some((Operator::Add, 3, Associativity::Left)),
        Symbol::Minus => Some((Operator::Subtract, 3, Associativity::Left)),
        Symbol::Multiply => Some((Operator::Multiply, 4, Associativity::Left)),
        Symbol::Divide => Some((Operator::Divide, 4, Associativity::Left)),
        _ => None,
    }
}

/// Parses an expression that can not be broken down further.
fn parse_expr_atom<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Expr> {
    let (token, span) = match run.read_next_where(is_atom, deny) {
        ReadResult::Some(token, span) => (token, span),
        ReadResult::None(_) => return ParseResult::Error,
    };

    match token.symbol {
        Symbol::Number(number_str) => ParseResult::Success(
            Expr::NumericLiteral {
                value: number_str.parse().unwrap(),
                range: span.range(),
            },
            span.cont(),
        ),
        Symbol::LeftParen => {
            let start = span.range();

            let (expr, run) = match parse_expr(span.cont(), &deny.insert(Symbol::RightParen)) {
                ParseResult::Error => return ParseResult::Error,
                ParseResult::Success(expr, run) => (expr, run),
            };

            let span = match run.read_next_where(|token| token == &Symbol::RightParen, deny) {
                ReadResult::None(_) => return ParseResult::Error,
                ReadResult::Some(_, span) => span,
            };

            let end = span.range();

            ParseResult::Success(
                Expr::ParenthesizedExpression {
                    expr: Box::new(expr),
                    range: start.extend(end),
                },
                span.cont(),
            )
        }
        Symbol::Identifier(ident) if span.peek_symbol() == Some(&Symbol::LeftParen) => {
            let start = span.range();

            let mut run = span.cont().read_next().unwrap_run();
            let mut args = Vec::new();

            let deny = deny.insert(Symbol::Comma);

            while let ParseResult::Success(arg, next) = parse_expr(run.clone(), &deny) {
                args.push(arg);
                run = next;
            }

            let span = match run.read_next_where(|token| token == &Symbol::RightParen, deny) {
                ReadResult::None(_) => return ParseResult::Error,
                ReadResult::Some(_, span) => span,
            };

            let end = span.range();

            ParseResult::Success(
                Expr::FunctionCall {
                    function: ident.to_string(),
                    args,
                    range: start.extend(end),
                },
                span.cont(),
            )
        }
        Symbol::Identifier(ident) => ParseResult::Success(
            Expr::VariableValue {
                identifier: ident.to_string(),
                range: span.range(),
            },
            span.cont(),
        ),
        _ => unreachable!(),
    }
}

/// Parses an expression using the precedence climbing method.
///
/// See: https://eli.thegreenplace.net/2012/08/02/parsing-expressions-by-precedence-climbing
///
fn parse_expr_chain<'a>(run: Run<'a>, deny: &Deny<'a>, min_prec: u8) -> ParseResult<'a, Expr> {
    let (mut lhs, mut run) = match parse_expr_atom(run, deny) {
        ParseResult::Success(expr, next) => (expr, next),
        ParseResult::Error => return ParseResult::Error,
    };

    loop {
        let (result, span) = match run.read_next_where_some(is_some_operation, deny) {
            ReadResult::None(_) => return ParseResult::Error,
            ReadResult::Some(result, span) => (result, span),
        };

        let (operator, prec, assoc) = result;

        if prec < min_prec {
            run = span.cont();
            break;
        }

        run = span.cont();

        let new_min_prec = if assoc == Associativity::Left {
            prec + 1
        } else {
            prec
        };

        let rhs = match parse_expr_chain(run.clone(), deny, new_min_prec) {
            ParseResult::Error => break,
            ParseResult::Success(rhs, next) => {
                run = next;
                rhs
            }
        };

        let range = lhs.range().extend(rhs.range());

        lhs = Expr::BinaryExpression {
            left_hand_side: Box::new(lhs),
            right_hand_side: Box::new(rhs),
            operator: operator,
            range,
        }
    }

    ParseResult::Success(lhs, run)
}

pub fn parse_expr<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Expr> {
    parse_expr_chain(run, deny, 0)
}
