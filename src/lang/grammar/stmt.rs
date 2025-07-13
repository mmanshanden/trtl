use crate::lang::{grammar::expr::parse_expr, parse::{Deny, ParseResult, Parser}, run::Span, Marker, Markers, ReadResult, Run, Stmt, Token};

fn into_markers<'a, 'b>(
    span: &'b Span<'a>,
    append: impl FnOnce(Token<'a>) -> Marker<'a>,
) -> Markers<'a> {
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


fn parse_stmt_scope<'a>() -> impl Parser<'a, Stmt> {
    |run: Run<'a>, deny: Deny<'a>| {
        let parser = wrapped(
            Token::LeftBrace,
            Token::RightBrace,
            many(parse_stmt()).map(Stmt::Scope),
        );

        match parser.parse(run, deny) {
            ParseResult::Error => ParseResult::Error,
            ParseResult::Success(result, markers, run) => {
                ParseResult::Success(result, markers, run)
            }
        }
    }
}

fn parse_stmt_expr<'a>() -> impl Parser<'a, Stmt> {
    first(
        parse_expr().deny(Token::SemiColon).map(Stmt::Expr),
        expect(Token::SemiColon, Marker::Plain),
    )
}

pub fn parse_stmt_move<'a>() -> impl Parser<'a, Stmt> {
    combine(
        expect_pred(
            |&t| matches!(t, Token::Forward | Token::Left | Token::Right),
            Marker::Move,
        ),
        first(
            parse_expr().deny(Token::SemiColon),
            expect(Token::SemiColon, Marker::Plain),
        ),
        |stmt, expr| match stmt {
            Token::Forward => Stmt::Forward(expr),
            Token::Left => Stmt::Left(expr),
            Token::Right => Stmt::Right(expr),
            _ => unreachable!(),
        },
    )
}

pub fn parse_stmt_if<'a>() -> impl Parser<'a, Stmt> {
    let parse_if = second(
        expect(Token::If, Marker::Keyword),
        combine(
            parse_expr().deny(Token::LeftBrace),
            parse_stmt_scope().deny(Token::Else),
            |expr, stmt| (expr, stmt),
        ),
    );

    let parse_else = |run: Run<'a>, deny: Deny<'a>| {
        let is_else = |&token| token == Token::Else;
        let deny_any = |t: &Token| t.dist() > 0;

        let span = match run.first_where(is_else, deny_any) {
            ReadResult::None(run) => return ParseResult::Success(None, Vec::new(), run),
            ReadResult::Some(_, span) => span,
        };

        let markers = span.dissect(
            Token::is_whitespace,
            |t| match t {
                Token::Whitespace(ws) => Marker::Whitespace(ws),
                Token::Comment(cmt) => Marker::Comment(cmt),
                Token::LineBreak => Marker::LineBreak,
                _ => unreachable!(),
            },
            |t| Marker::UnexpectedToken {
                expected: span.last().unwrap(),
                actual: t,
            },
            |t| Marker::Keyword(t),
        );

        match parse_stmt_scope().parse(span.next(), deny) {
            ParseResult::Error => ParseResult::Error,
            ParseResult::Success(stmt, stmt_markers, next) => {
                ParseResult::Success(Some(stmt), [markers, stmt_markers].concat(), next)
            }
        }
    };

    combine(
        parse_if.debug("if_parser"),
        parse_else,
        |(if_expr, if_body), else_block| {
            if let Some(else_body) = else_block {
                Stmt::IfElse(if_expr, Box::new(if_body), Box::new(else_body))
            } else {
                Stmt::If(if_expr, Box::new(if_body))
            }
        },
    )
}

fn parse_stmt_while<'a>() -> impl Parser<'a, Stmt> {
    second(
        expect(Token::While, Marker::Keyword),
        combine(
            parse_expr().deny(Token::LeftBrace),
            parse_stmt_scope().map(Box::new),
            Stmt::While,
        ),
    )
}

fn parse_stmt_return<'a>() -> impl Parser<'a, Stmt> {
    first(
        combine(
            expect(Token::Return, Marker::Flow),
            parse_expr().optional(),
            |_, expr| Stmt::Return(expr),
        ),
        expect(Token::SemiColon, Marker::Plain),
    )
}


// pub fn parse_expr_unit<'a>() -> impl Parser<'a, Expr> {
//     let is_expr_token = |token: &'a Token<'a>| match token {
//         Token::Number(_) => true,
//         Token::Identifier(_) => true,
//         Token::LeftParen => true,
//         _ => false,
//     };

//     move |run: Run<'a>, deny: Deny<'a>| {
//         let (token, span) = match run.first_where(is_expr_token, deny.clone()) {
//             ReadResult::Some(token, span) => (token, span),
//             ReadResult::None(_) => return ParseResult::Error,
//         };

//         match token {
//             Token::Number(number_str) => {
//                 let number = number_str.parse().unwrap();
//                 let number = Expr::Num(number);
                
//                 let markers = into_markers(&span, |_| {
//                     return Marker::Number(number_str);
//                 });

//                 ParseResult::Success(number, markers, span.next())
//             }
//             Token::LeftParen => {
//                 let mut markers = into_markers(&span, |token| {
//                     return Marker::Plain(token);
//                 });

//                 let deny = deny.insert(Token::RightParen);

//                 let (expr, run) = match parse_expr().parse(span.next(), deny) {
//                     ParseResult::Error => return ParseResult::Error,
//                     ParseResult::Success(expr, mut output, run) => {
//                         markers.append(&mut output);
//                         (expr, run)
//                     }
//                 };

//                 ParseResult::Success(expr, markers, run)
//             }
//             Token::Identifier(ident) => {
//                 let mut markers = into_markers(&span, |_| {
//                     return Marker::Identifier(ident);
//                 });

//                 if let Some(Token::LeftParen) = span.peek() {
//                     match seperated_by(parse_expr(), Token::Comma).parse(span.next(), deny) {
//                         ParseResult::Error => return ParseResult::Error,
//                         ParseResult::Success(args, mut inner_markers, run) => {
//                             markers.append(&mut inner_markers);

//                             let expr = Expr::Call(ident.to_string(), args);
//                             ParseResult::Success(expr, markers, run)
//                         }
//                     }
//                 } else {
//                     let ident = ident.to_string();
//                     let ident = Expr::Var(ident);

//                     ParseResult::Success(ident, markers, span.next())
//                 }
//             }
//             _ => unreachable!(),
//         }
//     }
// }

fn parse_stmt<'a>() -> impl Parser<'a, Stmt> {
    let is_stmt_token = |token: &Token<'a>| match token {
        Token::LeftBrace => true,
        Token::If => true,
        Token::While => true,
        Token::Forward => true,
        Token::Left => true,
        Token::Right => true,
        Token::Return => true,
        Token::LeftParen => true,
        Token::Identifier(_) => true,
        Token::Number(_) => true,
        _ => false,
    };

    move |run: Run<'a>, deny: Deny<'a>| {
        let (token, span) = match run.first_where(is_stmt_token, deny.clone()) {
            ReadResult::Some(token, span) => (token, span),
            ReadResult::None(_) => return ParseResult::Error
        };

        match token {
            Token::LeftBrace => {
                let mut markers = into_markers(&span, |token| {
                    return Marker::Plain(token);
                });

                let mut run = span.next().clone();
                let mut stmts = Vec::new();

                loop {
                    match parse_stmt().parse(run.clone(), deny.clone().insert(Token::RightBrace)) {
                        ParseResult::Error => break,
                        ParseResult::Success(stmt, mut output, next) => {
                            markers.append(&mut output);
                            stmts.push(stmt);

                            run = next;
                        }
                    };
                }

                let run = match run.first_where(|token| token == &Token::RightBrace, deny) {
                    ReadResult::None(_) => return ParseResult::Error,
                    ReadResult::Some(_, span) => {
                        let mut output = into_markers(&span, Marker::Plain);
                        markers.append(&mut output);
                        span.next()
                    }
                };

                return  ParseResult::Success(Stmt::Scope(stmts), markers, run);
            }
            Token::Left => {
                let mut markers = into_markers(&span, |token| {
                    return Marker::Move(token);
                });

                let mut run = span.next();
                
                match parse_expr().parse(run, deny.insert(Token::SemiColon)) {

                }
            }
            
            _ => unreachable!(),
        };
    }
}