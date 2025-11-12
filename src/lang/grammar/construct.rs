use crate::lang::{
    Cons, Param, ReadResult, Run, Stmt, Symbol, grammar::statement::is_stmt_symbol,
    parse::ParseResult, run::Deny,
};

fn is_cons_symbol(symbol: &Symbol<'_>) -> bool {
    match symbol {
        Symbol::Func => true,
        any if is_stmt_symbol(any) => true,
        _ => false,
    }
}

fn parse_param<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Param> {
    let (param, reading) = match run.read_until_some(Symbol::identifier, deny) {
        ReadResult::Some(param, reading) => (param, reading),
        ReadResult::None(_) => return ParseResult::Error,
    };

    let param = Param {
        name: param.to_string(),
        index_from: reading.index_from(),
        index_to: reading.index_to(),
    };

    ParseResult::Success(param, reading.resume())
}

fn parse_param_list_delimited<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Vec<Param>> {
    let reading = match run.consume_if_true(|&symbol| symbol == Symbol::LeftParen) {
        ReadResult::Some(_, reading) => reading,
        ReadResult::None(_) => return ParseResult::Error,
    };

    let deny = deny.insert(Symbol::RightParen);
    let run = reading.resume();

    // first parameter
    let (params, run) = match parse_param(run.clone(), &deny) {
        ParseResult::Error => (Vec::new(), run),
        ParseResult::Success(param, run) => {
            let mut params = vec![param];
            let mut run = run;

            let deny = deny.insert(Symbol::Comma);

            // subsequent arguments
            loop {
                let reading = match run
                    .clone()
                    .read_until_true(|&symbol| symbol == Symbol::Comma, &deny)
                {
                    ReadResult::Some(_, reading) => reading,
                    ReadResult::None(revert) => {
                        run = revert;
                        break;
                    }
                };

                match parse_param(reading.resume(), &deny) {
                    ParseResult::Error => break,
                    ParseResult::Success(arg, next) => {
                        params.push(arg);
                        run = next;
                    }
                };
            }

            (params, run)
        }
    };

    let reading = match run.read_until_true(|&symbol| symbol == Symbol::RightParen, deny) {
        ReadResult::None(_) => return ParseResult::Error,
        ReadResult::Some(_, reading) => reading,
    };

    ParseResult::Success(params, reading.resume())
}

fn parse_cons<'a>(run: Run<'a>, deny: &Deny<'a>) -> ParseResult<'a, Cons> {
    let (token, reading) = match run.read_until_true(is_cons_symbol, deny) {
        ReadResult::Some(token, reading) => (token, reading),
        ReadResult::None(_) => return ParseResult::Error,
    };

    match token.symbol {
        Symbol::Func => {
            let (name, reading) = match reading
                .resume()
                .read_until_some(Symbol::identifier, &deny.insert(Symbol::LeftBrace))
            {
                ReadResult::Some(name, reading) => (name, reading),
                ReadResult::None(_) => return ParseResult::Error,
            };

            let (params, run) = match parse_param_list_delimited(reading.resume(), deny) {
                ParseResult::Error => return ParseResult::Error,
                ParseResult::Success(params, run) => (params, run),
            };

            ParseResult::Success(
                Cons::Func {
                    name: name.to_string(),
                    parmeters: params,
                    body: Stmt::Block {
                        body: Vec::new(),
                        index_from: 0,
                        index_to: 0,
                    },
                    index_from: 0,
                    index_to: 0,
                },
                run,
            )
        }
        _ => unreachable!(),
    }
}
