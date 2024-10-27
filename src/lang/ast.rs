

#[derive(Debug)]
pub enum Expr {
    Assign(Box<Expr>, Box<Expr>),

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

#[derive(Debug)]
pub enum Stmt {
    If(Expr, Box<Stmt>),
    IfElse(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
    Forward(Expr),
    Left(Expr),
    Right(Expr),
    Scope(Vec<Stmt>),
    Expr(Expr),
    Return(Option<Expr>)
}

#[derive(Debug)]
pub enum Entry {
    Func(String, Vec<String>, Stmt),
    Stmt(Stmt)
}

pub type Program = Vec<Entry>;
