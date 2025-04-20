
use crate::lang::lex::Token;

use super::ast::Entry;
use super::ast::Expr;
use super::ast::Program;
use super::ast::Stmt;
use super::run::ParseResult;
use super::run::Run;
use super::run::ParseResult::Ok;
use super::run::ParseResult::Err;
use super::run::Stack;


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

/// Parses an expression wrapped by `(` and `)` tokens.
/// 
/// Returns a parse result containing the parsed expression or an error
/// when parsing was not possible.
/// 
/// This function is used to parse sub expressions. 
fn parse_expr_parens<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Expr> {
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
/// See: https://eli.thegreenplace.net/2012/08/02/parsing-expressions-by-precedence-climbing
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

/// Parses an expression
/// 
fn parse_expr<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Expr> {
    parse_expr_1(run, deny, 0)
}


#[macro_export]
macro_rules! take_best_parse {
    [$result:expr] => {
        $result
    };
    [$result1:expr, $result2:expr] => {
        match ($result1, $result2) {
            (Ok(t1, run1), Ok(t2, run2)) => {
                if run1.dist <= run2.dist {
                    Ok(t1, run1)
                } else {
                    Ok(t2, run2)
                }
            }
            (r1, Err(_)) => r1,
            (Err(_), r2) => r2,
        }
    };
    [$result1:expr, $( $tail:expr ),+] => {
        take_best_parse!($result1, take_best_parse!($($tail),+))
    };
}

fn parse_stmt_while<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
    let n = match run.first_where(|token| token == Token::While, deny) {
        None => return Err(run),
        Some(n) => n,
    };

    let run = run.advance_by(n);

    let (expr, run) = match parse_expr(run, deny) {
        Err(run) => return ParseResult::Err(run),
        Ok(expr, next) => (expr, next),
    };

    let (stmt, run) = match parse_stmt(run, deny) {
        Err(run) => return Err(run),
        Ok(stmt, next) => (Box::new(stmt), next),
    };
    
    Ok(Stmt::While(expr, stmt), run)
}

fn parse_stmt_if<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
    let n = match run.first_where(|token| token == Token::If, deny) {
        None => return Err(run),
        Some(n) => n,
    };

    let run = run.advance_by(n);

    let (expr, run) = match parse_expr(run, deny) {
        Err(run) => return ParseResult::Err(run),
        Ok(expr, next) => (expr, next),
    };

    deny.push(Token::Else);

    let parse_stmt_result = parse_stmt(run, deny);
   
    deny.pop();

    let (if_part, run) = match parse_stmt_result {
        Err(run) => return Err(run),
        Ok(stmt, next) => (Box::new(stmt), next),
    };

    if run.token() == Token::Else {
        let run = run.advance();

        let (else_part, run) = match parse_stmt(run, deny) {
            Err(run) => return ParseResult::Err(run),
            Ok(stmt, next) => (Box::new(stmt), next),
        };

        Ok(Stmt::IfElse(expr, if_part, else_part), run)
    } else {
        Ok(Stmt::If(expr, if_part), run)
    }
}

fn parse_stmt_block<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
    let block_delimiter = |token| match token {
        Token::LeftBrace => Some(Token::LeftBrace),
        Token::RightBrace => Some(Token::RightBrace),
        _ => None,
    };

    let mut run = match run.first_where_some(block_delimiter, deny) {
        Some((n, Token::LeftBrace)) => run.advance_by(n),
        Some((_, Token::RightBrace)) => run.insert(Token::LeftBrace),
        _ => return Err(run),
    };

    let mut stmts = Vec::new();

    deny.push(Token::RightBrace);

    loop {
        match parse_stmt(run, deny) {
            Ok(stmt, next) => {
                stmts.push(stmt);
                run = next;
            }
            Err(errored) => {
                run = errored;
                break;
            }
        }
    }

    deny.pop();

    let run = match run.first_where(|token| token == Token::RightBrace, deny) {
        Some(n) => run.advance_by(n),
        None => run.insert(Token::RightBrace),
    };

    Ok(Stmt::Scope(stmts), run)
}

fn parse_stmt_forward<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
    let n = match run.first_where(|token| token == Token::Forward, &deny) {
        Some(n) => n,
        None => return Err(run)
    };

    let run = run.advance_by(n);

    deny.push(Token::SemiColon);

    let parse_expr_result = parse_expr(run, deny);

    deny.pop();

    let (expr, run) = match parse_expr_result {
        Err(errored) => return Err(errored),
        Ok(expr, next) => (expr, next),
    };

    let run = match run.first_where(|token| token == Token::SemiColon, deny) {
        Some(n) => run.advance_by(n),
        None => run.insert(Token::SemiColon),
    };

    Ok(Stmt::Forward(expr), run)
}

fn parse_stmt_left<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
    let n = match run.first_where(|token| token == Token::Left, &deny) {
        Some(n) => n,
        None => return Err(run)
    };

    let run = run.advance_by(n);

    deny.push(Token::SemiColon);

    let parse_expr_result = parse_expr(run, deny);

    deny.pop();

    let (expr, run) = match parse_expr_result {
        Err(errored) => return Err(errored),
        Ok(expr, next) => (expr, next),
    };

    let run = match run.first_where(|token| token == Token::SemiColon, deny) {
        Some(n) => run.advance_by(n),
        None => run.insert(Token::SemiColon),
    };

    Ok(Stmt::Left(expr), run)
}

fn parse_stmt_right<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
    let n = match run.first_where(|token| token == Token::Right, &deny) {
        Some(n) => n,
        None => return Err(run)
    };

    let run = run.advance_by(n);

    deny.push(Token::SemiColon);

    let parse_expr_result = parse_expr(run, deny);

    deny.pop();

    let (expr, run) = match parse_expr_result {
        Err(errored) => return Err(errored),
        Ok(expr, next) => (expr, next),
    };

    let run = match run.first_where(|token| token == Token::SemiColon, deny) {
        Some(n) => run.advance_by(n),
        None => run.insert(Token::SemiColon),
    };

    Ok(Stmt::Right(expr), run)
}

fn parse_stmt_expr<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
    deny.push(Token::SemiColon);

    let parse_expr_result = parse_expr(run, deny);

    deny.pop();

    let (expr, run) = match parse_expr_result {
        Err(errored) => return Err(errored),
        Ok(expr, next) => (expr, next),
    };

    let run = match run.first_where(|token| token == Token::SemiColon, deny) {
        Some(n) => run.advance_by(n),
        None => run.insert(Token::SemiColon),
    };

    Ok(Stmt::Expr(expr), run)
}

fn parse_stmt_return<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
    let n = match run.first_where(|token| token == Token::Return, deny) {
        Some(n) => n,
        None => return Err(run)
    };

    let run = run.advance_by(n);

    deny.push(Token::SemiColon);

    let parse_expr_result = parse_expr(run, deny);

    deny.pop();

    let (expr, run) = match parse_expr_result {
        Err(run) => (None, run),
        Ok(expr, next) => (Some(expr), next),
    };

    let run = match run.first_where(|token| token == Token::SemiColon, deny) {
        Some(n) => run.advance_by(n),
        None => run.insert(Token::SemiColon),
    };

    Ok(Stmt::Return(expr), run)
}

fn parse_stmt<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
    take_best_parse![
        parse_stmt_block(run.clone(), deny),
        parse_stmt_while(run.clone(), deny),
        parse_stmt_if(run.clone(), deny),
        parse_stmt_forward(run.clone(), deny),
        parse_stmt_left(run.clone(), deny),
        parse_stmt_right(run.clone(), deny),
        parse_stmt_expr(run.clone(), deny),
        parse_stmt_return(run.clone(), deny)
    ]

    // let is_stmt_token = |token| -> Option<Token> {
    //     match token {
    //         Token::If => Some(token),
    //         Token::LeftBrace => Some(token),
    //         Token::RightBrace => Some(token),
    //         _ => None,
    //     }
    // };

    // match run.first_where_some(is_stmt_token, deny) {
    //     Some((_, Token::If)) => return parse_stmt_if(run, deny),
    //     Some((_, Token::LeftBrace)) => return parse_stmt_block(run, deny),
    //     Some((_, Token::RightBrace)) => return parse_stmt_block(run, deny),
    //     _ => return parse_stmt_expr(run, deny),
    // }
}


fn parse_func_params<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Vec<String>> {
    if run.token() != Token::LeftParen {
        return Err(run);
    }

    let mut run = run.advance();
    let mut params = Vec::new();

    deny.push(Token::RightParen);

    // parse first parameter
    if let Some((n, identifier)) = run.first_where_some(|token| token.identifier(), deny) {
        params.push(identifier.to_string());
        run = run.advance_by(n);

        // parse commas followed by more parameters
        while let Some(n) = run.first_where(|t| t == Token::Comma, deny) {
            let inter_run = run.clone().advance_by(n);

            deny.push(Token::Comma);

            match inter_run.first_where_some(|token| token.identifier(), deny) {
                None => break,
                Some((n, ident)) => {
                    params.push(ident.to_string());
                    run = inter_run.advance_by(n);
                }
            }

            deny.pop(); // pop Comma
        }
    }

    deny.pop(); // pop RightParen

    let run = match run.first_where(|t| t == Token::RightParen, deny) {
        Some(n) => run.advance_by(n),
        None => run.insert(Token::RightParen),
    };

    Ok(params, run)
}

fn parse_entry_func<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Entry> {
    let n = match run.first_where(|token| token == Token::Func, deny) {
        None => return Err(run),
        Some(n) => n,
    };

    let run = run.advance_by(n);

    let (n, identifier) = match run.first_where_some(|token| token.identifier(), deny) {
        None => return Err(run),
        Some((n, identifier)) => (n, identifier),
    };

    let run = run.advance_by(n);

    let (params, run) = match parse_func_params(run, deny) {
        Err(errored) => return Err(errored),
        Ok(params, run) => (params, run),
    };

    let (body, run) = match parse_stmt_block(run, deny) {
        Err(run) => return Err(run),
        Ok(stmt, next) => (stmt, next),
    };

    Ok(Entry::Func(identifier.to_string(), params, body), run)
}

fn parse_entry_stmt<'a>(mut run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Entry> {
    deny.push(Token::Func);

    let parse_stmt_result = parse_stmt(run, deny);

    deny.pop();

    match parse_stmt_result {
        Err(err) => Err(err),
        Ok(stmt, next) => Ok(Entry::Stmt(stmt), next)
    }
}

pub fn parse_program<'a>(mut run: Run<'a>, ) -> ParseResult<'a, Program> {
    let mut program = Vec::new();
    let mut deny = Vec::new();

    loop {
        let result = take_best_parse![
            parse_entry_stmt(run.clone(), &mut deny), 
            parse_entry_func(run.clone(), &mut deny)
        ];

        match result {
            Err(errored) => {
                run = errored;
                break;
            }
            Ok(stmt, next) => {
                program.push(stmt);
                run = next;
            }
        }
    }

    Ok(program, run)
}

