

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
    Num(f32),
    Call(String, Vec<Expr>),
}

#[derive(Debug)]
pub enum Stmt {
    If(Expr, Box<Stmt>),
    IfElse(Expr, Box<Stmt>, Box<Stmt>),
    Scope(Vec<Stmt>),
    Expr(Expr),
    Func(String, Vec<String>, Box<Stmt>),
}

