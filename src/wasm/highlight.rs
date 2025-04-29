use crate::lang::ast::{parse_program, Lexer, ParseResult, Parser, Run, Tag, Token};

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
        Token::Eof => "EOF".to_string(),
        Token::Undefined(str) => str.to_string(),
        token => unreachable!("uexpected token: \"{:?}\"", token)
    }
}

pub fn highlight(input: &str) -> Highlight {
    let tokens = Lexer::new(input).tokens();
    let tags = match parse_program().parse(Run::new(&tokens), Vec::new()) {
        ParseResult::Error => panic!("error parsing the input"),
        ParseResult::Success(_, tags, _) => tags
    };

    println!("tags: {:?}", tags);

    let mut lines = Vec::new();
    let mut line = Vec::new();

    for tag in tags {
        let (value, kind) = match tag {
            Tag::LineBreak => {
                line.push(Fragment {
                    value: String::new(),
                    hint: 0,
                    kind: 0
                });
                
                lines.push(line.clone());
                line.clear();
                continue;
            },

            Tag::Whitespace(str) => (vec![str.to_string()], 0),
            Tag::Plain(token) => (vec![token_to_string(token)], 0),
            Tag::Comment(str) => (vec![str.to_string()], 1),
            Tag::Call(name) => (vec![name.to_string()], 2),
            Tag::Move(token) => (vec![token_to_string(token)], 3),
            Tag::Identifier(name) => (vec![name.to_string()], 4),
            Tag::Number(num) => (vec![num   .to_string()], 5),
            Tag::Keyword(token) => (vec![token_to_string(token)], 6),

            Tag::UnexpectedToken { expected: _, actual } => {
                (actual.iter().map(|&token| token_to_string(token)).collect(), 20)
            }
        };

        for value in value {
            line.push(Fragment {
                value,
                hint: 0,
                kind
            });
        }
    }

    line.push(Fragment {
        value: String::new(),
        hint: 0,
        kind: 0
    });
    
    lines.push(line);
    
    Highlight {
        lines,
        hints: Vec::new()
    }
}