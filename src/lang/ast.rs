use std::collections::{HashMap, HashSet};

use crate::lang::run::Range;

use super::{Symbol, lex::Tokens};

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

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        value: f64,
        range: Range,
    },
    Parenthesis {
        expr: Box<Expr>,
        range: Range,
    },
    Variable {
        identifier: String,
        range: Range,
    },
    Call {
        function: String,
        args: Vec<Expr>,
        range: Range,
    },
    Binary {
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
            Expr::Literal { range, .. } => *range,
            Expr::Parenthesis { range, .. } => *range,
            Expr::Variable { range, .. } => *range,
            Expr::Call { range, .. } => *range,
            Expr::Binary { range, .. } => *range,
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
pub enum Cons {
    Func {
        name: String,
        parmeters: Vec<String>,
        body: Stmt,
    },
    Statement {
        stmt: Stmt,
    },
}

pub type Program = Vec<Cons>;
