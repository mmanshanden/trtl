use std::collections::{HashMap, HashSet};

use crate::lang::run::Range;

use super::{Symbol, lex::Tokens};

#[derive(Debug, Clone)]
pub struct Span {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
pub enum MoveDirection {
    MoveForward,
    TurnLeft,
    TurnRight,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Operator {
    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterEqualThan,
    LessThan,
    LessEqualThan,
}

#[derive(Debug, Clone)]
pub enum Expr {
    NumericLiteral {
        value: f64,
        range: Range,
    },
    ParenthesizedExpression {
        expr: Box<Expr>,
        range: Range,
    },
    VariableValue {
        identifier: String,
        range: Range,
    },
    FunctionCall {
        function: String,
        args: Vec<Expr>,
        range: Range,
    },
    BinaryExpression {
        left_hand_side: Box<Expr>,
        right_hand_side: Box<Expr>,
        operator: Operator,
        range: Range,
    },
    // Assign(String, Box<Expr>),

    // Equals(Box<Expr>, Box<Expr>),
    // NotEqual(Box<Expr>, Box<Expr>),
    // GreaterThan(Box<Expr>, Box<Expr>),
    // GreaterEqualThan(Box<Expr>, Box<Expr>),
    // LessThan(Box<Expr>, Box<Expr>),
    // LessEqualThan(Box<Expr>, Box<Expr>),

    // Add(Box<Expr>, Box<Expr>),
    // Sub(Box<Expr>, Box<Expr>),
    // Mul(Box<Expr>, Box<Expr>),
    // Div(Box<Expr>, Box<Expr>),

    // Var(String),
    // Num(f64),
    // Call(String, Vec<Expr>),
}

impl Expr {
    pub fn range(&self) -> Range {
        match self {
            Expr::NumericLiteral { range, .. } => *range,
            Expr::ParenthesizedExpression { range, .. } => *range,
            Expr::VariableValue { range, .. } => *range,
            Expr::FunctionCall { range, .. } => *range,
            Expr::BinaryExpression { range, .. } => *range,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    If {
        condition: Expr,
        body: Box<Stmt>,
        alternate: Option<Box<Stmt>>,
        range: Range,
    },
    Move {
        argument: Expr,
        direction: MoveDirection,
        range: Range,
    },
    Block {
        body: Vec<Stmt>,
        range: Range,
    },
    Expression {
        expr: Expr,
        range: Range,
    }, // If(Expr, Box<Stmt>),
       // IfElse(Expr, Box<Stmt>, Box<Stmt>),
       // While(Expr, Box<Stmt>),
       // Forward(Expr),
       // Left(Expr),
       // Right(Expr),
       // Scope(Vec<Stmt>),
       // Expr(Expr),
       // Return(Option<Expr>),
}

#[derive(Debug, Clone)]
pub enum Entry {
    Func(String, Vec<String>, Stmt),
    Stmt(Stmt),
}

pub type Program = Vec<Entry>;

#[derive(Debug, Clone)]
pub enum Marker<'a> {
    Number(&'a str),
    Identifier(&'a str),
    Function(&'a str),
    Flow(Symbol<'a>),
    Call(&'a str),
    Keyword(Symbol<'a>),
    Plain(Symbol<'a>),
    Move(Symbol<'a>),
    Comment(&'a str),
    Whitespace(&'a str),
    LineBreak,

    UnexpectedToken {
        expected: Symbol<'a>,
        actual: Tokens<'a>,
    },
}

pub type Markers<'a> = Vec<Marker<'a>>;
