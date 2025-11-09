use crate::lang::{
    Expr, ReadResult, Run, Stmt, Symbol,
    grammar::expression::{is_expr_symbol, parse_expr},
    parse::ParseResult,
    run::{Deny, Span},
};

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

fn parse_stmt<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Stmt> {
    let index_from = run.position();

    let (token, span) = match run.read_until_true(is_stmt_symbol, deny) {
        ReadResult::Some(token, span) => (token, span),
        ReadResult::None(_) => return ParseResult::Error,
    };

    match token.symbol {
        Symbol::LeftBrace => {
            let deny = deny.insert(Symbol::RightBrace);

            let mut run = span.cont();
            let mut body = Vec::new();

            while let ParseResult::Success(stmt, next) = parse_stmt(run.clone(), &deny) {
                body.push(stmt);
                run = next;
            }

            let span = match run.read_until_true(|&token| token == Symbol::RightBrace, deny) {
                ReadResult::None(_) => return ParseResult::Error,
                ReadResult::Some(_, span) => span,
            };

            let index_to = span.index_to();

            ParseResult::Success(
                Stmt::Block {
                    body,
                    index_from,
                    index_to,
                },
                span.cont(),
            )
        }
        Symbol::LeftParen | Symbol::Identifier(_) | Symbol::Number(_) => {
            let deny = deny.insert(Symbol::SemiColon);

            let (expr, run) = match parse_expr(span.revert(), &deny) {
                ParseResult::Error => return ParseResult::Error,
                ParseResult::Success(expr, next) => (expr, next),
            };

            let span = match run.read_until_true(|&token| token == Symbol::SemiColon, deny) {
                ReadResult::None(_) => return ParseResult::Error,
                ReadResult::Some(_, span) => span,
            };

            let index_to = span.index_to();

            ParseResult::Success(
                Stmt::Expression {
                    expr,
                    index_from,
                    index_to,
                },
                span.cont(),
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
        lexer.tokens()
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
