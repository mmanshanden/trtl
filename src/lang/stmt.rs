use crate::lex::Token;

use super::{expr::{parse_expr, Expr}, ParseResult, Run, Stack};
use ParseResult::{Err, Ok};


#[derive(Debug)]
pub enum Stmt {
    If(Expr, Box<Stmt>),
    IfElse(Expr, Box<Stmt>, Box<Stmt>),
    Scope(Vec<Stmt>),
    Expr(Expr),
    Func(String, Vec<String>, Box<Stmt>),
}

#[macro_export]
macro_rules! take_best_parse {
    [$result:expr] => {
        $result
    };
    [$result1:expr, $result2:expr] => {
        match ($result1, $result2) {
            (Ok(t1, run1), Ok(t2, run2)) => {
                if run1.dist < run2.dist {
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
    let mut run = match run.first_where_some(
        |token| match token {
            Token::LeftBrace => Some(Token::LeftBrace),
            Token::RightBrace => Some(Token::RightBrace),
            _ => None,
        },
        deny,
    ) {
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

fn parse_func_params<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Vec<String>> {
    if run.token() != Token::LeftParen {
        return Err(run);
    }

    let mut run = run.advance();
    let mut params = Vec::new();

    deny.push(Token::RightParen);

    // parse first parameter
    if let Some((n, ident)) = run.first_where_some(|token| token.identifier(), deny) {
        params.push(ident.to_string());
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

fn parse_func<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
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

    let (body, run) = match parse_stmt(run, deny) {
        Err(run) => return Err(run),
        Ok(stmt, next) => (Box::new(stmt), next),
    };

    Ok(Stmt::Func(identifier.to_string(), params, body), run)
}

fn parse_stmt<'a>(run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Stmt> {
    return take_best_parse![
        parse_stmt_if(run.clone(), deny),
        parse_stmt_block(run.clone(), deny),
        parse_stmt_expr(run.clone(), deny)
    ];

    let is_stmt_token = |token| -> Option<Token> {
        match token {
            Token::If => Some(token),
            Token::LeftBrace => Some(token),
            Token::RightBrace => Some(token),
            _ => None,
        }
    };

    match run.first_where_some(is_stmt_token, deny) {
        Some((_, Token::If)) => return parse_stmt_if(run, deny),
        Some((_, Token::LeftBrace)) => return parse_stmt_block(run, deny),
        Some((_, Token::RightBrace)) => return parse_stmt_block(run, deny),
        _ => return parse_stmt_expr(run, deny),
    }
}

pub fn parse_program<'a>(mut run: Run<'a>, deny: &mut Stack<'a>) -> ParseResult<'a, Vec<Stmt>> {
    let mut program = Vec::new();

    loop {
        let result = take_best_parse![
            parse_stmt(run.clone(), deny), 
            parse_func(run.clone(), deny)
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

