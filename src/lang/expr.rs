use crate::lex::Token;

use super::{ParseResult, Run, Stack};
use ParseResult::{Err, Ok};


#[derive(Debug)]
pub enum Expr {
    Assign(Box<Expr>, Box<Expr>),

    Equals(Box<Expr>, Box<Expr>),
    NotEqual(Box<Expr>, Box<Expr>),
    GreaterThan(Box<Expr>, Box<Expr>),
    GreaterEqualThan(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    LessEqualThan(Box<Expr>, Box<Expr>),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    Var(String),
    Num(f32),
    Call(String, Vec<Expr>),
}

/// Parses a primary part of the expression.
///
/// A primary part can either be a:
///  - constanst value,
///  - variable,
///  - function call or,
///  - a sub expression wrapped by a `(` and `)` token.
///
/// Returns a parse result containing the parsed expression, or an error
/// when parsing was not possible.
///
fn parse_expr_primary<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Expr> {
    println!("parse_expr_primary, {:?}", run);
    let is_primary = |token: Token<'a>| match token {
        Token::Number(_) => Some(token),
        Token::Identifier(_) => Some(token),
        Token::LeftParen => Some(token),
        _ => None,
    };

    match run.first_where_some(is_primary, deny) {
        // Constant number case
        Some((n, Token::Number(num))) => {
            let run = run.advance_by(n);
            let num = num.parse().unwrap();
            let num = Expr::Num(num);

            Ok(num, run)
        }

        // Identifier, which can be either a var or a function call
        // when followed by a left parenthesis.
        Some((n, Token::Identifier(id))) => {
            let run = run.advance_by(n);
            let id = id.to_string();

            // An identifier that is immediately followed by an `(`
            // token is a function call.
            if run.token() == Token::LeftParen {
                return match parse_expr_args(run, deny) {
                    Err(run) => Err(run),
                    Ok(args, run) => {
                        let call = Expr::Call(id, args);
                        Ok(call, run)
                    }
                };
            }

            let id = Expr::Var(id);
            Ok(id, run)
        }

        // Sub expression case
        Some((_, Token::LeftParen)) => parse_expr_parens(run, deny),
        _ => Err(run),
    }
}

/// Parses a list of epxression arguments.
///
/// A list of expression arguments it a comma seperated list of expression
/// wrapped by `(` and `)` tokens.
///
/// Returns a parse result containg a vector of expressions or an error
/// when parsing was not possible.
///
fn parse_expr_args<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Vec<Expr>> {
    println!("parse_expr_args");
    if run.token() != Token::LeftParen {
        return Err(run);
    }

    let mut run = run.advance();
    let mut args = Vec::new();

    deny.push(Token::RightParen);
    deny.push(Token::Comma);

    // parse first argument
    match parse_expr(run, deny) {
        Err(next) => {
            return Err(next);
        }
        Ok(expr, next) => {
            args.push(expr);
            run = next;
        }
    }

    deny.pop(); // pop Comma

    // parse commas followed by arguments
    while let Some(n) = run.first_where(|t| t == Token::Comma, deny) {
        // inter_run saves the state of the current run in case we
        // reach a point where a comma is not followed by an expr.
        let inter_run = run.clone().advance_by(n);

        deny.push(Token::Comma);

        match parse_expr(inter_run, deny) {
            Ok(expr, next) => {
                args.push(expr);
                run = next;
            }
            Err(_) => break,
        }

        deny.pop(); // pop Comma
    }

    deny.pop(); // pop RightParen

    let run = match run.first_where(|t| t == Token::RightParen, deny) {
        Some(n) => run.advance_by(n),
        None => run.insert(Token::RightParen),
    };

    Ok(args, run)
}

fn parse_expr_parens<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Expr> {
    println!("parse_expr_parens");
    let n = match run.first_where(|t| t == Token::LeftParen, deny) {
        None => return Err(run),
        Some(n) => n,
    };

    let run = run.advance_by(n);

    deny.push(Token::RightParen);

    let (expr, run) = match parse_expr(run, deny) {
        Err(run) => return Err(run),
        Ok(expr, run) => (expr, run),
    };

    deny.pop();

    let run = match run.first_where(|t| t == Token::RightParen, deny) {
        Some(n) => run.advance_by(n),
        None => run.insert(Token::RightParen),
    };

    Ok(expr, run)
}

#[derive(PartialEq)]
enum Associativity {
    Left,
    Right,
}

/// The recurrent case for a given minimum precedence level
///
fn parse_expr_1<'a>(mut run: Run<'a>, deny: &mut Stack<'a>, min_prec: u8) -> ParseResult<'a, Expr> {
    let mut lhs = match parse_expr_primary(run, deny) {
        Err(run) => return Err(run),
        Ok(expr, next) => {
            run = next;
            expr
        }
    };

    let lhs_is_var = matches!(lhs, Expr::Var(_));

    let is_operation = |token| match &token {
        Token::Equals if min_prec < 2 => Some((token, 1, Associativity::Left)),
        Token::NotEqual if min_prec < 2 => Some((token, 1, Associativity::Left)),
        Token::LessThan if min_prec < 2 => Some((token, 1, Associativity::Left)),
        Token::LessEqualThan if min_prec < 2 => Some((token, 1, Associativity::Left)),
        Token::GreaterThan if min_prec < 2 => Some((token, 1, Associativity::Left)),
        Token::GreaterEqualThan if min_prec < 2 => Some((token, 1, Associativity::Left)),
        Token::Plus => Some((token, 2, Associativity::Left)),
        Token::Minus => Some((token, 2, Associativity::Left)),
        Token::Multiply => Some((token, 3, Associativity::Left)),
        Token::Divide => Some((token, 3, Associativity::Left)),
        Token::Assign if lhs_is_var => Some((token, 0, Associativity::Right)),
        _ => None,
    };

    while let Some((op, prec, assoc)) = is_operation(run.token()) {
        if prec < min_prec {
            break;
        }

        run = run.advance();

        let next_min_prec = if assoc == Associativity::Left {
            prec + 1
        } else {
            prec
        };

        let rhs = match parse_expr_1(run, deny, next_min_prec) {
            Err(errored) => {
                return Ok(lhs, errored);
            }
            Ok(expr, next) => {
                run = next;
                expr
            }
        };

        let lhs_boxxed = Box::new(lhs);
        let rhs_boxxed = Box::new(rhs);

        lhs = match op {
            Token::Assign => Expr::Assign(lhs_boxxed, rhs_boxxed),
            Token::Equals => Expr::Equals(lhs_boxxed, rhs_boxxed),
            Token::NotEqual => Expr::NotEqual(lhs_boxxed, rhs_boxxed),
            Token::GreaterThan => Expr::GreaterThan(lhs_boxxed, rhs_boxxed),
            Token::GreaterEqualThan => Expr::GreaterEqualThan(lhs_boxxed, rhs_boxxed),
            Token::LessThan => Expr::LessThan(lhs_boxxed, rhs_boxxed),
            Token::LessEqualThan => Expr::LessEqualThan(lhs_boxxed, rhs_boxxed),
            Token::Plus => Expr::Add(lhs_boxxed, rhs_boxxed),
            Token::Minus => Expr::Sub(lhs_boxxed, rhs_boxxed),
            Token::Multiply => Expr::Mul(lhs_boxxed, rhs_boxxed),
            Token::Divide => Expr::Div(lhs_boxxed, rhs_boxxed),
            _ => unreachable!(),
        }
    }

    ParseResult::Ok(lhs, run)
}

pub fn parse_expr<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Expr> {
    parse_expr_1(run, deny, 0)
}