use std::collections::HashMap;

use super::ast::{Entry, *};
use crate::machine::cpu::Op;

#[derive(Clone, Copy, Debug)]
enum Loc {
    Local(i32),
    Global(i32),
}

trait Scope: Copy {
    fn to_loc(self, loc: i32) -> Loc;
}

impl<F> Scope for F
where
    F: Copy + Fn(i32) -> Loc,
{
    fn to_loc(self, loc: i32) -> Loc {
        self(loc)
    }
}

type Env = HashMap<String, Loc>;
type Ptr = i32;
type Allocs = i32;

pub enum CompileError {
    UndefinedVariable(String),
    UndefinedAssignment,
    UnsupportedExpression,
}

type CompileResult = Result<(Vec<Op>, Ptr, Allocs), CompileError>;

fn compile_call(
    func: String,
    args: Vec<Expr>,
    env: &mut Env,
    ptr: Ptr,
    scope: impl Scope,
) -> CompileResult {
    let mut allocs = 0;
    let mut ptr = ptr;
    let mut code = Vec::new();

    let len = args.len() as i32;
    let name = func.as_str();

    for expr in args.into_iter().rev() {
        let (expr_code, expr_ptr, expr_allocs) = compile_expr(expr, env, ptr, scope)?;

        code.extend(expr_code);
        ptr += expr_ptr;
        allocs += expr_allocs;
    }

    match (name, len) {
        ("sin", 1) => {
            code.push(Op::Sin);
        }
        _ => {
            code.push(Op::Call(func));
            code.push(Op::Ajs(-len)); // clean up params
            code.push(Op::LoadRR); // load return value
        }
    }

    Ok((code, ptr, allocs))
}

fn compile_binary_expr(
    e1: Expr,
    e2: Expr,
    operand: Op,
    env: &mut Env,
    ptr: Ptr,
    scope: impl Scope,
) -> CompileResult {
    let (c1, ptr1, as1) = compile_expr(e1, env, ptr, scope)?;
    let (c2, ptr2, as2) = compile_expr(e2, env, ptr1, scope)?;

    let mut code = Vec::new();
    code.extend(c1);
    code.extend(c2);
    code.push(operand);

    Ok((code, ptr2, as1 + as2))
}

fn compile_expr(expr: Expr, env: &mut Env, ptr: Ptr, scope: impl Scope) -> CompileResult {
    return match expr {
        Expr::NumericLiteral { value, .. } => {
            let n = value.to_bits();
            Ok((vec![Op::PushF(n)], ptr, 0))
        }
        Expr::ParenthesizedExpression { expr, .. } => compile_expr(*expr, env, ptr, scope),
        Expr::BinaryExpression {
            left_hand_side: left,
            right_hand_side: right,
            operator,
            ..
        } => match operator {
            Operator::Assign => {
                let (mut code, mut ptr, mut allocs) = compile_expr(*right, env, ptr, scope)?;

                let identifier = match *left {
                    Expr::VariableValue { identifier: v, .. } => v,
                    _ => return Err(CompileError::UndefinedAssignment),
                };

                let loc = match env.get(&identifier) {
                    Some(l) => *l,
                    None => {
                        let loc = scope.to_loc(ptr);
                        env.insert(identifier, loc);

                        allocs = allocs + 1;
                        ptr = ptr + 1;

                        loc
                    }
                };

                code.push(match loc {
                    Loc::Global(a) => Op::StoreG(a),
                    Loc::Local(a) => Op::StoreL(a),
                });

                Ok((code, ptr, allocs))
            }
            Operator::Add => compile_binary_expr(*left, *right, Op::Add, env, ptr, scope),
            Operator::Subtract => compile_binary_expr(*left, *right, Op::Sub, env, ptr, scope),
            Operator::Multiply => compile_binary_expr(*left, *right, Op::Mul, env, ptr, scope),
            Operator::Divide => compile_binary_expr(*left, *right, Op::Div, env, ptr, scope),
            Operator::Equal => compile_binary_expr(*left, *right, Op::Eq, env, ptr, scope),
            Operator::NotEqual => compile_binary_expr(*left, *right, Op::Neq, env, ptr, scope),
            Operator::GreaterThan => compile_binary_expr(*left, *right, Op::Gt, env, ptr, scope),
            Operator::LessThan => compile_binary_expr(*left, *right, Op::Lt, env, ptr, scope),
            Operator::GreaterEqualThan => {
                compile_binary_expr(*left, *right, Op::Gte, env, ptr, scope)
            }
            Operator::LessEqualThan => compile_binary_expr(*left, *right, Op::Lte, env, ptr, scope),
        },
        Expr::VariableValue { identifier: v, .. } => match env.get(&v) {
            None => Err(CompileError::UndefinedVariable(v)),
            Some(l) => match l {
                Loc::Global(a) => Ok((vec![Op::LoadG(*a)], ptr, 0)),
                Loc::Local(a) => Ok((vec![Op::LoadL(*a)], ptr, 0)),
            },
        },
        Expr::FunctionCall {
            args,
            function: func,
            ..
        } => compile_call(func, args, env, ptr, scope),
    };
}

fn compile_seq(seq: Vec<Stmt>, env: &mut Env, ptr: Ptr, scope: impl Scope) -> CompileResult {
    let mut code = Vec::new();

    let mut inner_env = env.clone();
    let mut inner_ptr = ptr;
    let mut allocs = ptr;

    for stmt in seq {
        let (ops, p, a) = compile_stmt(stmt, &mut inner_env, inner_ptr, scope)?;

        code.extend(ops);

        inner_ptr = p;
        allocs = p.max(a);
    }

    Ok((code, ptr, allocs))
}

fn compile_if(cond: Expr, body: Stmt, env: &mut Env, ptr: Ptr, scope: impl Scope) -> CompileResult {
    let mut code = Vec::new();

    let (expr, ptr, a1) = compile_expr(cond, env, ptr, scope)?;
    let (body, _, a2) = compile_stmt(body, env, ptr, scope)?;

    code.extend(expr);
    code.push(Op::Brf(body.len() as i32));
    code.extend(body);

    Ok((code, ptr, a1 + a2))
}

fn compile_ifelse(
    cond: Expr,
    b1: Stmt,
    b2: Stmt,
    env: &mut Env,
    ptr: Ptr,
    scope: impl Scope,
) -> CompileResult {
    let mut code = Vec::new();

    let (e1, ptr, a1) = compile_expr(cond, env, ptr, scope)?;
    let (b1, _, a2) = compile_stmt(b1, env, ptr, scope)?;
    let (b2, _, a3) = compile_stmt(b2, env, ptr, scope)?;

    code.extend(e1);
    code.push(Op::Brf(1 + b1.len() as i32));
    code.extend(b1);
    code.push(Op::Bra(b2.len() as i32));
    code.extend(b2);

    Ok((code, ptr, i32::max(a1 + a2, a1 + a3)))
}

fn compile_while(
    cond: Expr,
    stmt: Stmt,
    env: &mut Env,
    ptr: Ptr,
    scope: impl Scope,
) -> CompileResult {
    let mut code = Vec::new();

    let (expr, ptr, a1) = compile_expr(cond, env, ptr, scope)?;
    let (body, _, a2) = compile_stmt(stmt, env, ptr, scope)?;

    code.extend(expr);
    code.push(Op::Brf(1 + body.len() as i32));
    code.extend(body);
    code.push(Op::Bra(code.len() as i32 * -1 - 1));

    Ok((code, ptr, a1 + a2))
}

fn compile_func(
    name: String,
    args: Vec<String>,
    stmt: Stmt,
    env: &mut Env,
    ptr: Ptr,
    scope: impl Scope,
) -> CompileResult {
    let mut code = Vec::new();
    let mut inner_env = env.clone();
    let mut inner_ptr = -3;

    for arg in args {
        let loc = scope.to_loc(inner_ptr);

        inner_env.insert(arg, loc);
        inner_ptr -= 1;
    }

    let (mut body, _, allocs) = compile_stmt(stmt, &mut inner_env, 0, Loc::Local)?;

    if body.last() != Some(&Op::Ret) {
        body.push(Op::Ret);
    }

    code.push(Op::Bra(body.len() as i32 + 2));
    code.push(Op::Label(name));
    code.push(Op::Ajs(allocs));
    code.extend(body);

    Ok((code, ptr, allocs))
}

fn compile_ret(expr: Option<Expr>, env: &mut Env, ptr: Ptr, scope: impl Scope) -> CompileResult {
    if let Some(expr) = expr {
        let (mut code, ptr, allocs) = compile_expr(expr, env, ptr, scope)?;

        code.push(Op::StoreRR);
        code.push(Op::Ret);

        return Ok((code, ptr, allocs));
    }

    Ok((vec![Op::Ret], ptr, 0))
}

fn compile_command(
    op: Op,
    expr: Expr,
    env: &mut Env,
    ptr: Ptr,
    scope: impl Scope,
) -> CompileResult {
    let mut code = Vec::new();

    let (expr, ptr, allocs) = compile_expr(expr, env, ptr, scope)?;

    code.extend(expr);
    code.push(op);

    Ok((code, ptr, allocs))
}

fn compile_expr_stmt(expr: Expr, env: &mut Env, ptr: Ptr, scope: impl Scope) -> CompileResult {
    let mut code = Vec::new();

    let (expr, ptr, allocs) = compile_expr(expr, env, ptr, scope)?;

    code.extend(expr);
    code.push(Op::Ajs(-1)); // pop expr result from stack as it won't be used anyway

    Ok((code, ptr, allocs))
}

fn compile_stmt(stmt: Stmt, env: &mut Env, ptr: Ptr, scope: impl Scope) -> CompileResult {
    match stmt {
        Stmt::Scope(seq) => compile_seq(seq, env, ptr, scope),
        Stmt::Return(expr) => compile_ret(expr, env, ptr, scope),
        Stmt::Expr(expr) => compile_expr_stmt(expr, env, ptr, scope),
        Stmt::If(cond, stmt) => compile_if(cond, *stmt, env, ptr, scope),
        Stmt::IfElse(cond, b1, b2) => compile_ifelse(cond, *b1, *b2, env, ptr, scope),
        Stmt::While(cond, stmt) => compile_while(cond, *stmt, env, ptr, scope),
        // Stmt::Print(expr) => compile_command(Op::Print, expr, env, ptr, scope),
        Stmt::Forward(expr) => compile_command(Op::Movf, expr, env, ptr, scope),
        Stmt::Left(expr) => compile_command(Op::Movl, expr, env, ptr, scope),
        Stmt::Right(expr) => compile_command(Op::Movr, expr, env, ptr, scope),
    }
}

fn compile_entry(entry: Entry, env: &mut Env, ptr: Ptr) -> CompileResult {
    match entry {
        Entry::Func(name, args, stmt) => compile_func(name, args, stmt, env, ptr, Loc::Local),
        Entry::Stmt(stmt) => compile_stmt(stmt, env, ptr, Loc::Global),
    }
}

pub fn compile(program: Program) -> Result<Vec<Op>, CompileError> {
    let mut code = Vec::new();
    let mut body = Vec::new();
    let mut env = Env::new();
    let mut ptr = 0;
    let mut allocs = 0;

    for entry in program {
        let (ops, p, a) = compile_entry(entry, &mut env, ptr)?;

        body.extend(ops);

        ptr = p;
        allocs = p.max(a);
    }

    code.push(Op::Ajs(allocs));
    code.extend(body);

    Ok(code)
}
