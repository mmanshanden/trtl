use std::collections::{HashMap, HashSet};

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
        index_from: usize,
        index_to: usize,
    },
    Parenthesis {
        expr: Box<Expr>,
        index_from: usize,
        index_to: usize,
    },
    Variable {
        identifier: String,
        index_from: usize,
        index_to: usize,
    },
    Call {
        function: String,
        args: Vec<Expr>,
        index_from: usize,
        index_to: usize,
    },
    Binary {
        left_hand_side: Box<Expr>,
        right_hand_side: Box<Expr>,
        operator: Operator,
        index_from: usize,
        index_to: usize,
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
    pub fn index_from(&self) -> usize {
        match self {
            Expr::Literal { index_from, .. } => *index_from,
            Expr::Parenthesis { index_from, .. } => *index_from,
            Expr::Variable { index_from, .. } => *index_from,
            Expr::Call { index_from, .. } => *index_from,
            Expr::Binary { index_from, .. } => *index_from,
        }
    }

    pub fn index_to(&self) -> usize {
        match self {
            Expr::Literal { index_to, .. } => *index_to,
            Expr::Parenthesis { index_to, .. } => *index_to,
            Expr::Variable { index_to, .. } => *index_to,
            Expr::Call { index_to, .. } => *index_to,
            Expr::Binary { index_to, .. } => *index_to,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    If {
        condition: Expr,
        body: Box<Stmt>,
        alternate: Option<Box<Stmt>>,
        index_from: usize,
        index_to: usize,
    },
    Move {
        argument: Expr,
        direction: MoveDirection,
        index_from: usize,
        index_to: usize,
    },
    Block {
        body: Vec<Stmt>,
        index_from: usize,
        index_to: usize,
    },
    Expression {
        expr: Expr,
        index_from: usize,
        index_to: usize,
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
pub struct Param {
    pub name: String,
    pub index_from: usize,
    pub index_to: usize,
}

#[derive(Debug, Clone)]
pub enum Cons {
    Func {
        name: String,
        parameters: Vec<Param>,
        body: Stmt,
        index_from: usize,
        index_to: usize,
    },
    Statement {
        stmt: Stmt,
        index_from: usize,
        index_to: usize,
    },
}

pub type Program = Vec<Cons>;
