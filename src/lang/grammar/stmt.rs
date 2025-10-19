use crate::lang::{
    Expr, Marker, Markers, ReadResult, Run, Stmt, Symbol,
    grammar::expr::parse_expr,
    parse::{Deny, ParseResult},
    run::Span,
};

fn is_stmt_symbol(symbol: &Symbol<'_>) -> bool {
    match symbol {
        Symbol::LeftBrace => true,
        Symbol::If => true,
        Symbol::While => true,
        Symbol::Forward => true,
        Symbol::Left => true,
        Symbol::Right => true,
        Symbol::Return => true,
        Symbol::LeftParen => true,
        Symbol::Identifier(_) => true,
        Symbol::Number(_) => true,
        _ => false,
    }
}

fn parse_stmt<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Stmt> {
    let (token, span) = match run.read_next_where(is_stmt_symbol, deny) {
        ReadResult::Some(token, span) => (token, span),
        ReadResult::None(_) => return ParseResult::Error,
    };

    let start = span.range();

    match token.symbol {
        Symbol::LeftBrace => {
            let deny = deny.insert(Symbol::RightBrace);

            let mut run = span.cont();
            let mut body = Vec::new();

            while let ParseResult::Success(stmt, next) = parse_stmt(run.clone(), &deny) {
                body.push(stmt);
                run = next;
            }

            let span = match run.read_next_where(|&token| token == Symbol::RightBrace, deny) {
                ReadResult::None(_) => return ParseResult::Error,
                ReadResult::Some(_, span) => span,
            };

            let end = span.range();

            ParseResult::Success(
                Stmt::Block {
                    body,
                    range: start.extend(end),
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

            let span = match run.read_next_where(|&token| token == Symbol::SemiColon, deny) {
                ReadResult::None(_) => return ParseResult::Error,
                ReadResult::Some(_, span) => span,
            };

            let end = span.range();

            ParseResult::Success(
                Stmt::Expression {
                    expr,
                    range: start.extend(end),
                },
                span.cont(),
            )
        }
        _ => unreachable!(),
    }
}
