use crate::lang::{parse_program, Lexer, Run, Marker, Token};


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
    pub hints: Vec<String>
} 
    

fn token_to_string(token: Token) -> String {
    match token {
        Token::Assign => "=".to_string(),
        Token::Plus => "+".to_string(),
        Token::Minus => "-".to_string(),
        Token::Multiply => "*".to_string(),
        Token::Divide => "/".to_string(),
        Token::Bang => "!".to_string(),
        Token::LessThan => "<".to_string(),
        Token::GreaterThan => ">".to_string(),
        Token::LessEqualThan => "<=".to_string(),
        Token::GreaterEqualThan => ">=".to_string(),
        Token::Equals => "==".to_string(),
        Token::NotEqual => "!=".to_string(),
        Token::If => "if".to_string(),
        Token::Else => "else".to_string(),
        Token::While => "while".to_string(),
        Token::Func => "func".to_string(),
        Token::Return => "return".to_string(),
        Token::Break => "break".to_string(),
        Token::Forward => "forward".to_string(),
        Token::Left => "left".to_string(),
        Token::Right => "right".to_string(),
        Token::Comma => ",".to_string(),
        Token::SemiColon => ";".to_string(),
        Token::LeftParen => "(".to_string(),
        Token::RightParen => ")".to_string(),
        Token::LeftBrace => "{".to_string(),
        Token::RightBrace => "}".to_string(),
        Token::Identifier(name) => name.to_string(),
        Token::Number(num) => num.to_string(),
        Token::Eof => "".to_string(),
        Token::Undefined(str) => str.to_string(),
        Token::True => "true".to_string(),
        Token::False => "false".to_string(),
        token => unreachable!("uexpected token: \"{:?}\"", token)
    }
}

pub fn highlight(input: &str) -> Highlight {
    let tokens = Lexer::new(input).tokens();
    let run = Run::new(&tokens);

    let tags = match parse_program(run) {
        Err(_) => panic!("error parsing the input"),
        Ok((_, tags)) => tags
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
            Marker::Keyword(token) => {
                line.push(Fragment {
                    value: token_to_string(token),
                    kind: 3,
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
            Marker::Move(token) => {
                line.push(Fragment {
                    value: token_to_string(token),
                    kind: 4,
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

            Marker::UnexpectedToken { expected: _, actual } => for &token in actual {
                line.push(Fragment {
                    value: token_to_string(token),
                    kind: 40,
                    hint: 0
                });
            }
        };
    }

    lines.push(line);
    
    Highlight {
        lines,
        hints: Vec::new()
    }
}