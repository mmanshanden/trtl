use crate::lang::{Expr, Operator, ReadResult, Run, Symbol, parse::ParseResult, run::Deny};

#[derive(PartialEq, Clone, Copy, Debug)]
enum Associativity {
    Left,
    Right,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Delimiter {
    Comma,
    End,
}

pub fn is_expr_symbol(token: &Symbol<'_>) -> bool {
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

fn is_arg_list_delimiter(symbol: &Symbol<'_>) -> Option<Delimiter> {
    match symbol {
        Symbol::Comma => Some(Delimiter::Comma),
        Symbol::RightParen => Some(Delimiter::End),
        _ => None,
    }
}

fn parse_arg_list_delimited<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Vec<Expr>> {
    let reading = match run.consume() {
        ReadResult::Some(_, reading) if reading.symbol() == &Symbol::LeftParen => reading,
        _ => return ParseResult::Error,
    };

    let deny = deny.insert(Symbol::RightParen);
    let run = reading.cont();

    // first argument
    let (args, run) = match parse_expr(run.clone(), &deny) {
        ParseResult::Error => (Vec::new(), run),
        ParseResult::Success(arg, run) => {
            let mut args = vec![arg];
            let mut run = run;

            let deny = deny.insert(Symbol::Comma);

            // subsequent arguments
            loop {
                let reading = match run
                    .clone()
                    .read_where_true(|&symbol| symbol == Symbol::Comma, &deny)
                {
                    ReadResult::Some(_, reading) => reading,
                    ReadResult::None(revert) => {
                        run = revert;
                        break;
                    }
                };

                match parse_expr(reading.cont(), &deny) {
                    ParseResult::Error => break,
                    ParseResult::Success(arg, next) => {
                        args.push(arg);
                        run = next;
                    }
                };
            }

            (args, run)
        }
    };

    let reading = match run.read_where_true(|&symbol| symbol == Symbol::RightParen, deny) {
        ReadResult::None(_) => return ParseResult::Error,
        ReadResult::Some(_, reading) => reading,
    };

    ParseResult::Success(args, reading.cont())
}

/// Parses an expression that can not be broken down further.
fn parse_expr_atom<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Expr> {
    let index_from = run.position();

    let reading = match run.read_where_true(is_expr_symbol, deny) {
        ReadResult::Some(_, reading) => reading,
        ReadResult::None(_) => return ParseResult::Error,
    };

    match reading.symbol() {
        Symbol::Number(number_str) => {
            let index_to = reading.index_to();

            return ParseResult::Success(
                Expr::Literal {
                    value: number_str.parse().unwrap(),
                    index_from,
                    index_to,
                },
                reading.cont(),
            );
        }
        Symbol::LeftParen => {
            let (expr, run) = match parse_expr(reading.cont(), &deny.insert(Symbol::RightParen)) {
                ParseResult::Error => return ParseResult::Error,
                ParseResult::Success(expr, run) => (expr, run),
            };

            let reading = match run.read_where_true(|token| token == &Symbol::RightParen, deny) {
                ReadResult::None(_) => return ParseResult::Error,
                ReadResult::Some(_, reading) => reading,
            };

            let index_to = reading.index_to();

            ParseResult::Success(
                Expr::Parenthesis {
                    expr: Box::new(expr),
                    index_from,
                    index_to,
                },
                reading.cont(),
            )
        }
        Symbol::Identifier(ident) if reading.peek_symbol() == Some(&Symbol::LeftParen) => {
            let (args, run) = match parse_arg_list_delimited(reading.cont(), &deny) {
                ParseResult::Error => return ParseResult::Error,
                ParseResult::Success(args, run) => (args, run),
            };

            let index_to = run.position();

            ParseResult::Success(
                Expr::Call {
                    function: ident.to_string(),
                    args,
                    index_from,
                    index_to,
                },
                run,
            )
        }
        Symbol::Identifier(ident) => {
            let index_to = reading.index_to();

            return ParseResult::Success(
                Expr::Variable {
                    identifier: ident.to_string(),
                    index_from,
                    index_to,
                },
                reading.cont(),
            );
        }
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

    let index_from = lhs.index_from();

    loop {
        let (result, reading) = match run.read_where_some(is_some_operation, deny) {
            ReadResult::None(run) => return ParseResult::Success(lhs, run),
            ReadResult::Some(result, reading) => (result, reading),
        };

        let (operator, prec, assoc) = result;

        if prec < min_prec {
            run = reading.revert();
            break;
        } else {
            run = reading.cont();
        }

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

        let index_to = rhs.index_to();

        lhs = Expr::Binary {
            left_hand_side: Box::new(lhs),
            right_hand_side: Box::new(rhs),
            operator: operator,
            index_from,
            index_to,
        }
    }

    ParseResult::Success(lhs, run)
}

pub fn parse_expr<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Expr> {
    parse_expr_chain(run, deny, 0)
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
    fn test_parse_number_literal() {
        let tokens = create_tokens("42");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        match parse_expr(run, &deny) {
            ParseResult::Success(expr, _) => {
                if let Expr::Literal { value, .. } = expr {
                    assert_eq!(value, 42.0);
                } else {
                    panic!("Expected Literal expression");
                }
            }
            ParseResult::Error => panic!("Failed to parse number literal"),
        }
    }

    #[test]
    fn test_parse_variable() {
        let tokens = create_tokens("x");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        match parse_expr(run, &deny) {
            ParseResult::Success(expr, _) => {
                if let Expr::Variable { identifier, .. } = expr {
                    assert_eq!(identifier, "x");
                } else {
                    panic!("Expected Variable expression");
                }
            }
            ParseResult::Error => panic!("Failed to parse variable"),
        }
    }

    #[test]
    fn test_parse_binary_operation() {
        let tokens = create_tokens("2 + 3");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        match parse_expr(run, &deny) {
            ParseResult::Success(expr, _) => {
                if let Expr::Binary { operator, .. } = expr {
                    assert_eq!(operator, Operator::Add);
                } else {
                    panic!("Expected Binary expression");
                }
            }
            ParseResult::Error => panic!("Failed to parse binary operation"),
        }
    }

    #[test]
    fn test_parse_parenthesized_expr() {
        let tokens = create_tokens("(42)");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        match parse_expr(run, &deny) {
            ParseResult::Success(expr, _) => {
                if let Expr::Parenthesis { expr, .. } = expr {
                    if let Expr::Literal { value, .. } = *expr {
                        assert_eq!(value, 42.0);
                    } else {
                        panic!("Expected Literal inside Parenthesis");
                    }
                } else {
                    panic!("Expected Parenthesis expression");
                }
            }
            ParseResult::Error => panic!("Failed to parse parenthesized expression"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let tokens = create_tokens("foo(1, 2)");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        match parse_expr(run, &deny) {
            ParseResult::Success(expr, _) => {
                if let Expr::Call { function, args, .. } = expr {
                    assert_eq!(function, "foo");
                    assert_eq!(args.len(), 2);
                } else {
                    panic!("Expected Call expression");
                }
            }
            ParseResult::Error => panic!("Failed to parse function call"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        let tokens = create_tokens("2 + 3 * 4");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        match parse_expr(run, &deny) {
            ParseResult::Success(expr, _) => {
                if let Expr::Binary {
                    operator: op1,
                    right_hand_side,
                    ..
                } = expr
                {
                    assert_eq!(op1, Operator::Add);

                    if let Expr::Binary { operator: op2, .. } = *right_hand_side {
                        assert_eq!(op2, Operator::Multiply);
                    } else {
                        panic!("Expected multiplication in right hand side");
                    }
                } else {
                    panic!("Expected Binary expression");
                }
            }
            ParseResult::Error => panic!("Failed to parse expression with operator precedence"),
        }
    }

    #[test]
    fn test_assignment_expression() {
        let tokens = create_tokens("x = 42");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        match parse_expr(run, &deny) {
            ParseResult::Success(expr, _) => {
                if let Expr::Binary { operator, .. } = expr {
                    assert_eq!(operator, Operator::Assign);
                } else {
                    panic!("Expected Binary expression for assignment");
                }
            }
            ParseResult::Error => panic!("Failed to parse assignment expression"),
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let tokens = create_tokens("a = (b + 3) * 4 - foo(2, x)");
        let run = Run::new(&tokens);
        let deny = Deny::new();

        let expr = match parse_expr(run, &deny) {
            ParseResult::Success(expr, _) => expr,
            ParseResult::Error => panic!("Failed to parse expression"),
        };

        let target = Expr::Binary {
            left_hand_side: Box::new(Expr::Variable {
                identifier: "a".to_string(),
                index_from: 0,
                index_to: 1,
            }),
            right_hand_side: Box::new(Expr::Binary {
                left_hand_side: Box::new(Expr::Binary {
                    left_hand_side: Box::new(Expr::Parenthesis {
                        expr: Box::new(Expr::Binary {
                            left_hand_side: Box::new(Expr::Variable {
                                identifier: "b".to_string(),
                                index_from: 5,
                                index_to: 6,
                            }),
                            right_hand_side: Box::new(Expr::Literal {
                                value: 3.0,
                                index_from: 8,
                                index_to: 10,
                            }),
                            operator: Operator::Add,
                            index_from: 5,
                            index_to: 10,
                        }),
                        index_from: 3,
                        index_to: 11,
                    }),
                    right_hand_side: Box::new(Expr::Literal {
                        value: 4.0,
                        index_from: 13,
                        index_to: 15,
                    }),
                    operator: Operator::Multiply,
                    index_from: 3,
                    index_to: 15,
                }),
                right_hand_side: Box::new(Expr::Call {
                    function: "foo".to_string(),
                    args: vec![
                        Expr::Literal {
                            value: 2.0,
                            index_from: 20,
                            index_to: 21,
                        },
                        Expr::Variable {
                            identifier: "x".to_string(),
                            index_from: 22,
                            index_to: 24,
                        },
                    ],
                    index_from: 17,
                    index_to: 25,
                }),
                operator: Operator::Subtract,
                index_from: 3,
                index_to: 25,
            }),
            operator: Operator::Assign,
            index_from: 0,
            index_to: 25,
        };

        assert_eq!(expr, target);
    }
}
