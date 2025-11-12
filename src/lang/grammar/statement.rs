use crate::lang::{
    Expr, ReadResult, Run, Stmt, Symbol,
    grammar::expression::{is_expr_symbol, parse_expr},
    parse::ParseResult,
    run::{Deny, Reading},
};

// let match statements always return runs instead of readings

pub fn is_stmt_symbol(symbol: &Symbol<'_>) -> bool {
    match symbol {
        Symbol::LeftBrace => true,
        Symbol::If => true,
        Symbol::While => true,
        Symbol::Forward => true,
        Symbol::Left => true,
        Symbol::Right => true,
        Symbol::Return => true,
        any if is_expr_symbol(any) => true,
        _ => false,
    }
}

fn parse_condition<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Expr> {
    let run = match run.consume_if_true(|&symbol| symbol == Symbol::LeftParen) {
        ReadResult::Some(_, reading) => reading.resume(),
        _ => return ParseResult::Error,
    };

    let deny = deny.insert(Symbol::RightParen);

    let (expr, run) = match parse_expr(run, &deny) {
        ParseResult::Success(expr, next) => (expr, next),
        ParseResult::Error => return ParseResult::Error,
    };

    let run = match run.consume() {
        ReadResult::Some(token, reading) if token.symbol == Symbol::LeftParen => reading.resume(),
        _ => return ParseResult::Error,
    };

    ParseResult::Success(expr, run)
}

fn parse_list_of_stmts<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Vec<Stmt>> {
    let mut stmts = Vec::new();
    let mut run = run;

    // parse first statement
    if let ParseResult::Success(stmt, next) = parse_stmt(run.clone(), deny) {
        stmts.push(stmt);
        run = next;
    }

    let deny = deny.insert(Symbol::SemiColon);

    // parse subsequent statements
    while let ParseResult::Success(stmt, next) = parse_stmt(run.clone(), &deny) {
        stmts.push(stmt);
        run = next;
    }

    ParseResult::Success(stmts, run)
}

fn parse_stmt<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Stmt> {
    let index_from = run.position();

    let (token, reading) = match run.read_until_true(is_stmt_symbol, deny) {
        ReadResult::Some(token, reading) => (token, reading),
        ReadResult::None(_) => return ParseResult::Error,
    };

    match token.symbol {
        Symbol::LeftBrace => {
            let deny = deny.insert(Symbol::RightBrace);

            let (body, run) = match parse_list_of_stmts(reading.resume(), &deny) {
                ParseResult::Error => return ParseResult::Error,
                ParseResult::Success(body, next) => (body, next),
            };

            let index_to = run.position();

            ParseResult::Success(
                Stmt::Block {
                    body,
                    index_from,
                    index_to,
                },
                run,
            )
        }
        Symbol::If => {
            let (condition, run) = match parse_condition(reading.resume(), deny) {
                ParseResult::Error => return ParseResult::Error,
                ParseResult::Success(expr, next) => (expr, next),
            };

            let (body, run) = match parse_stmt(run, &deny.insert(Symbol::Else)) {
                ParseResult::Error => return ParseResult::Error,
                ParseResult::Success(stmt, next) => (stmt, next),
            };

            let (run, alternate) = match run.consume_if_true(|&symbol| symbol == Symbol::Else) {
                ReadResult::Some(_, reading) => {
                    let (else_stmt, run) = match parse_stmt(reading.resume(), deny) {
                        ParseResult::Error => return ParseResult::Error,
                        ParseResult::Success(stmt, run) => (stmt, run),
                    };

                    (run, Some(Box::new(else_stmt)))
                }
                ReadResult::None(run) => (run, None),
            };

            let index_to = run.position();

            ParseResult::Success(
                Stmt::If {
                    condition,
                    body: Box::new(body),
                    alternate,
                    index_from,
                    index_to,
                },
                run,
            )
        }
        Symbol::LeftParen | Symbol::Identifier(_) | Symbol::Number(_) => {
            let deny = deny.insert(Symbol::SemiColon);

            let (expr, run) = match parse_expr(reading.revert(), &deny) {
                ParseResult::Error => return ParseResult::Error,
                ParseResult::Success(expr, next) => (expr, next),
            };

            let run = match run.read_until_true(|&token| token == Symbol::SemiColon, deny) {
                ReadResult::None(_) => return ParseResult::Error,
                ReadResult::Some(_, reading) => reading.resume(),
            };

            let index_to = run.position();

            ParseResult::Success(
                Stmt::Expression {
                    expr,
                    index_from,
                    index_to,
                },
                run,
            )
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::{Lexer, lex::Token};

    use super::*;

    fn create_tokens<'a>(input: &'a str) -> Vec<Token<'a>> {
        let mut lexer = Lexer::new(input);
        lexer.collect()
    }

    #[test]
    fn test_parse_block_stmt() {
        let tokens = create_tokens("{}");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        match parse_stmt(run, &deny) {
            ParseResult::Success(stmt, _) => {
                if let Stmt::Block { body, .. } = stmt {
                    assert_eq!(body.len(), 0);
                } else {
                    panic!("Expected block statement");
                }
            }
            ParseResult::Error => panic!("Failed to parse empty block"),
        }
    }

    #[test]
    fn test_parse_expr_stmt() {
        let tokens = create_tokens("42;");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        match parse_stmt(run, &deny) {
            ParseResult::Success(stmt, _) => {
                if let Stmt::Expression { expr, .. } = stmt {
                    if let Expr::Literal { value, .. } = expr {
                        assert_eq!(value, 42.0);
                    } else {
                        panic!("Expected number expression");
                    }
                } else {
                    panic!("Expected expression statement");
                }
            }
            ParseResult::Error => panic!("Failed to parse expression statement"),
        }
    }

    #[test]
    fn test_parse_stmt_error() {
        // Test missing semicolon
        let tokens = create_tokens("42");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        match parse_stmt(run, &deny) {
            ParseResult::Error => (),
            _ => panic!("Expected error for missing semicolon"),
        }
    }
}
