use crate::lang::{
    Cons, ReadResult, Run, Symbol, grammar::statement::is_stmt_symbol, parse::ParseResult,
    run::Deny,
};

fn is_cons_symbol(symbol: &Symbol<'_>) -> bool {
    match symbol {
        Symbol::Func => true,
        any if is_stmt_symbol(any) => true,
        _ => false,
    }
}

fn parse_cons<'a>(mut run: Run<'a>, deny: &Deny) -> ParseResult<'a, Cons> {
    let (token, span) = match run.until(is_cons_symbol, deny) {
        ReadResult::Some(token, span) => (token, span),
        ReadResult::None(_) => return ParseResult::Error,
    };

    let start = span.range();

    match token.symbol {
        Symbol::Func => {
            let (name_token, span) =
                match run.until(is_cons_symbol, &deny.insert(Symbol::LeftBrace)) {
                    ReadResult::Some(token, span) => (token, span),
                    ReadResult::None(_) => return ParseResult::Error,
                };

            let name = match name_token.symbol {
                Symbol::Identifier(name) => name.to_string(),
                _ => unreachable!(),
            };

            let mut args = Vec::new();

            let span = match run.until(|&token| token == Symbol::LeftBrace, deny) {
                ReadResult::None(_) => return ParseResult::Error,
                ReadResult::Some(_, span) => span,
            };

            loop {
                let (arg, span) = match run.until_some(
                    Symbol::identifier,
                    deny.insert(Symbol::Comma).insert(Symbol::RightParen),
                ) {
                    ReadResult::Some(arg, span) => (arg.to_string(), span),
                    ReadResult::None(_) => break,
                };

                run = span.cont();
            }

            let end = span.range();

            ParseResult::Success(
                Cons::Func {
                    name,
                    parmeters: args,
                    range: start.extend(end),
                },
                span.cont(),
            )
        }
        _ => unreachable!(),
    }
}
