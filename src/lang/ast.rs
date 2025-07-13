use std::collections::{HashMap, HashSet};

use super::{Token, lex::Tokens};

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(String, Box<Expr>),

    Equals(Box<Expr>, Box<Expr>),
    NotEqual(Box<Expr>, Box<Expr>),
    GreaterThan(Box<Expr>, Box<Expr>),
    GreaterEqualThan(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    LessEqualThan(Box<Expr>, Box<Expr>),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    Var(String),
    Num(f64),
    Call(String, Vec<Expr>),
}

impl Expr {
    pub fn var(&self) -> Option<&String> {
        match self {
            Self::Var(var) => Some(var),
            _ => None,
        }
    }

    pub fn num(&self) -> Option<&f64> {
        match self {
            Self::Num(num) => Some(num),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    If(Expr, Box<Stmt>),
    IfElse(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
    Forward(Expr),
    Left(Expr),
    Right(Expr),
    Scope(Vec<Stmt>),
    Expr(Expr),
    Return(Option<Expr>),
}

#[derive(Debug, Clone)]
pub enum Entry {
    Func(String, Vec<String>, Stmt),
    Stmt(Stmt),
}

pub type Program = Vec<Entry>;

#[derive(Debug, Clone, Copy)]
pub enum Scope {
    Global,
    Local,
}

#[derive(Debug, Clone)]
pub struct Context {
    variables: HashMap<String, Scope>,
    functions: HashSet<(String, usize)>,
    scope: Scope,
}

impl Context {
    pub fn new() -> Context {
        Self {
            variables: HashMap::new(),
            functions: HashSet::new(),
            scope: Scope::Global,
        }
    }

    pub fn set_scope(self, scope: Scope) -> Self {
        Self {
            variables: self.variables,
            functions: self.functions,
            scope: scope,
        }
    }

    pub fn register_variable(mut self, variable: impl Into<String>) -> Self {
        self.variables.insert(variable.into(), self.scope);

        Self {
            variables: self.variables,
            functions: self.functions,
            scope: self.scope,
        }
    }

    pub fn register_function(
        mut self,
        function: impl Into<String>,
        args: Vec<impl Into<String>>,
    ) -> Self {
        self.functions.insert((function.into(), args.len()));

        for arg in args {
            self.variables.insert(arg.into(), Scope::Local);
        }

        Self {
            variables: self.variables,
            functions: self.functions,
            scope: self.scope,
        }
    }

    pub fn is_known_variable(&self, variable: impl Into<String>) -> Option<&Scope> {
        let variable = variable.into();
        return self.variables.get(&variable);
    }

    pub fn is_known_function(&self, function_name: impl Into<String>, arg_count: usize) -> bool {
        let function_name = function_name.into();
        return self.functions.contains(&(function_name, arg_count));
    }
}

#[derive(Debug, Clone)]
pub enum Marker<'a> {
    Number(&'a str),
    Identifier(&'a str),
    Function(&'a str),
    Flow(Token<'a>),
    Call(&'a str),
    Keyword(Token<'a>),
    Plain(Token<'a>),
    Move(Token<'a>),
    Comment(&'a str),
    Whitespace(&'a str),
    LineBreak,

    UnexpectedToken {
        expected: Token<'a>,
        actual: Tokens<'a>,
    },
}

pub type Markers<'a> = Vec<Marker<'a>>;
