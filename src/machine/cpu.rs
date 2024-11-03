use std::collections::HashMap;

use super::canvas::Canvas;
use super::turtle::Turtle;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Op {
    Label(String),
    Ajs(i32),
    PushF(u64),
    Goto(String),
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Neq,
    Sin,
    Cos,
    StoreL(i32),
    LoadL(i32),
    StoreG(i32),
    LoadG(i32),
    Call(String),
    Ret,
    StoreRR,
    LoadRR,
    Brf(i32),
    Bra(i32),
    Print,
    Movf,
    Movl,
    Movr,
}

pub struct Cpu {
    stdout: fn(&str) -> (),
    turtle: Turtle,
    labels: HashMap<String, u64>,
    code: Vec<Op>,
    halted: bool,
    stack: [u64; 4096],
    ip: u64,
    sp: u64,
    fp: u64,
    rr: u64,
}

impl Cpu {
    pub fn new(stdout: fn(&str) -> (), code: Vec<Op>) -> Self {
        let mut labels = HashMap::new();

        for (i, c) in code.iter().enumerate() {
            if let Op::Label(l) = c {
                labels.insert(l.to_string(), i as u64);
            }
        }

        Self {
            stdout,
            turtle: Turtle::new(),
            labels,
            code,
            halted: false,
            stack: [0; 4096],
            ip: 0,
            sp: 0,
            fp: 0,
            rr: 0,
        }
    }

    fn pop_float(&mut self) -> f64 {
        let bits = self.pop();
        f64::from_bits(bits)
    }

    fn push_float(&mut self, val: f64) {
        let bits = val.to_bits();
        self.push(bits);
    }

    fn pop(&mut self) -> u64 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    fn push(&mut self, val: u64) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn run(&mut self, canvas: &mut Canvas) {
        if self.ip as usize == self.code.len() {
            self.halted = true;
            return;
        }

        // get current operation
        let op = self.code[self.ip as usize].clone();

        // advance instruction pointer
        self.ip += 1;

        match op {
            Op::PushF(f) => {
                self.push(f);
            }
            Op::Add => {
                let b = self.pop_float();
                let a = self.pop_float();
                self.push_float(a + b);
            }
            Op::Sub => {
                let b = self.pop_float();
                let a = self.pop_float();
                self.push_float(a - b);
            }
            Op::Mul => {
                let b = self.pop_float();
                let a = self.pop_float();
                self.push_float(a * b);
            }
            Op::Div => {
                let b = self.pop_float();
                let a = self.pop_float();
                self.push_float(a / b);
            }
            Op::Sin => {
                let val = self.pop_float().to_radians().sin();
                self.push_float(val);
            }
            Op::Gt => {
                let b = self.pop_float();
                let a = self.pop_float();
                if a > b {
                    self.push_float(1.0);
                } else {
                    self.push_float(0.0);
                }
            },
            Op::Lt => {
                let b = self.pop_float();
                let a = self.pop_float();
                if a < b {
                    self.push_float(1.0);
                } else {
                    self.push_float(0.0);
                }
            }
            Op::Eq => {
                let b = self.pop_float();
                let a = self.pop_float();
                if (a - b).abs() < f64::EPSILON {
                    self.push_float(1.0);
                } else {
                    self.push_float(0.0);
                }
            }
            Op::Neq => {
                let b = self.pop_float();
                let a = self.pop_float();
                if (a - b).abs() > f64::EPSILON {
                    self.push_float(1.0);
                } else {
                    self.push_float(0.0);
                }
            }
            Op::Ajs(n) => {
                self.sp = (self.sp as i32 + n) as u64;
            }
            Op::Bra(n) => {
                self.ip = (self.ip as i32 + n) as u64;
            }
            Op::Label(_) => {}
            Op::LoadG(adr) => {
                let val = self.stack[adr as usize];
                self.push(val);
            }
            Op::StoreG(adr) => {
                let val = self.stack[self.sp as usize - 1];
                self.stack[adr as usize] = val;
            }
            Op::LoadL(adr) => {
                let a = self.fp as i32 + adr;
                let val = self.stack[a as usize];
                self.push(val);
            }
            Op::StoreL(adr) => {
                let adr = self.fp as i32 + adr;
                let val = self.stack[self.sp as usize - 1];
                self.stack[adr as usize] = val;
            }
            Op::Call(l) => {
                self.push(self.ip);
                self.push(self.fp);
                self.fp = self.sp;
                self.ip = match self.labels.get(&l) {
                    Some(&ip) => ip,
                    None => self.code.len() as u64
                }
            }
            Op::Ret => {
                self.sp = self.fp;
                self.fp = self.pop();
                self.ip = self.pop();
            }
            Op::StoreRR => {
                self.rr = self.pop();
            }
            Op::LoadRR => {
                self.push(self.rr);
            }
            Op::Brf(n) => {
                if self.pop() == 0 {
                    self.ip = (self.ip as i32 + n) as u64;
                }
            }
            Op::Print => {
                let val = self.pop_float();
                let msg = format!("{}", val);
                (self.stdout)(&msg);
            }
            Op::Movf => {
                let x1 = self.turtle.x as i32;
                let y1 = self.turtle.y as i32;
                let d = self.pop_float();

                self.turtle.forward(d);
                let x2 = self.turtle.x as i32;
                let y2 = self.turtle.y as i32;
                canvas.draw_line(x1, y1, x2, y2, 0, 0, 0);
            }
            Op::Movl => {
                let a = self.pop_float();
                self.turtle.left(a);
            }
            Op::Movr => {
                let a = self.pop_float();
                self.turtle.right(a);
            }
            _ => panic!("Operation {:?} not implemented", op),
        };
    }

    pub fn run_n(&mut self, canvas: &mut Canvas, mut n: u32) {
        while n > 0 && !self.is_halted() {
            self.run(canvas);
            n -= 1;
        }
    }
}
