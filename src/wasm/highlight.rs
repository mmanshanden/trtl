use crate::lang::{Lexer, Marker, Run, Symbol, parse_program};

#[derive(Debug, Clone)]
pub struct Fragment {
    pub value: String,
    pub hint: u8,
    pub kind: u8,
}

pub type Line = Vec<Fragment>;

#[derive(Debug, Clone)]
pub struct Highlight {
    pub lines: Vec<Line>,
    pub hints: Vec<String>,
}

fn token_to_string(token: Symbol) -> String {
    match token {
        Symbol::Assign => "=".to_string(),
        Symbol::Plus => "+".to_string(),
        Symbol::Minus => "-".to_string(),
        Symbol::Multiply => "*".to_string(),
        Symbol::Divide => "/".to_string(),
        Symbol::Bang => "!".to_string(),
        Symbol::LessThan => "<".to_string(),
        Symbol::GreaterThan => ">".to_string(),
        Symbol::LessEqualThan => "<=".to_string(),
        Symbol::GreaterEqualThan => ">=".to_string(),
        Symbol::Equals => "==".to_string(),
        Symbol::NotEqual => "!=".to_string(),
        Symbol::If => "if".to_string(),
        Symbol::Else => "else".to_string(),
        Symbol::While => "while".to_string(),
        Symbol::Func => "func".to_string(),
        Symbol::Return => "return".to_string(),
        Symbol::Break => "break".to_string(),
        Symbol::Forward => "forward".to_string(),
        Symbol::Left => "left".to_string(),
        Symbol::Right => "right".to_string(),
        Symbol::Comma => ",".to_string(),
        Symbol::SemiColon => ";".to_string(),
        Symbol::LeftParen => "(".to_string(),
        Symbol::RightParen => ")".to_string(),
        Symbol::LeftBrace => "{".to_string(),
        Symbol::RightBrace => "}".to_string(),
        Symbol::Identifier(name) => name.to_string(),
        Symbol::Number(num) => num.to_string(),
        Symbol::Eof => "".to_string(),
        Symbol::Undefined(str) => str.to_string(),
        Symbol::True => "true".to_string(),
        Symbol::False => "false".to_string(),
        token => unreachable!("unexpected token: \"{:?}\"", token),
    }
}

pub fn highlight(input: &str) -> Highlight {
    let tokens = Lexer::new(input).collect();
    let run = Run::new(&tokens);

    let tags = match parse_program(run) {
        Err(_) => panic!("error parsing the input"),
        Ok((_, tags)) => tags,
    };

    let mut lines = Vec::new();
    let mut line = Vec::new();

    for tag in tags {
        match tag {
            Marker::LineBreak => {
                lines.push(line);
                line = Vec::new();
            }
            Marker::Number(str) => {
                line.push(Fragment {
                    value: str.to_string(),
                    kind: 1,
                    hint: 0,
                });
            }
            Marker::Identifier(str) => {
                line.push(Fragment {
                    value: str.to_string(),
                    kind: 2,
                    hint: 0,
                });
            }
            Marker::Keyword(token) => {
                line.push(Fragment {
                    value: token_to_string(token),
                    kind: 3,
                    hint: 0,
                });
            }
            Marker::Flow(token) => {
                line.push(Fragment {
                    value: token_to_string(token),
                    kind: 4,
                    hint: 0,
                });
            }
            Marker::Move(token) => {
                line.push(Fragment {
                    value: token_to_string(token),
                    kind: 5,
                    hint: 0,
                });
            }
            Marker::Function(str) => {
                line.push(Fragment {
                    value: str.to_string(),
                    kind: 10,
                    hint: 0,
                });
            }
            Marker::Call(str) => {
                line.push(Fragment {
                    value: str.to_string(),
                    kind: 11,
                    hint: 0,
                });
            }
            Marker::Plain(token) => {
                line.push(Fragment {
                    value: token_to_string(token),
                    kind: 0,
                    hint: 0,
                });
            }
            Marker::Comment(str) => {
                line.push(Fragment {
                    value: str.to_string(),
                    kind: 20,
                    hint: 0,
                });
            }
            Marker::Whitespace(str) => {
                line.push(Fragment {
                    value: str.to_string(),
                    kind: 0,
                    hint: 0,
                });
            }
            Marker::UnexpectedToken { expected: _, actual } => {
                for &token in actual {
                    line.push(Fragment {
                        value: token_to_string(token),
                        kind: 40,
                        hint: 0,
                    });
                }
            }
        };
    }

    lines.push(line);

    Highlight { lines, hints: Vec::new() }
}
