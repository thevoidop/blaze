#![allow(dead_code, unused_variables)]
use crate::ast::{Expr, Stmt, Token};
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Array(Vec<Value>),
    Null,
    Void,
    Function {
        params: Vec<crate::ast::Param>,
        body: Vec<Stmt>,
    },
}

enum Flow {
    Break,
    Continue,
    Return(Value),
}

fn to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

pub struct Interpreter {
    environments: Vec<HashMap<String, Value>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environments: vec![HashMap::new()],
        }
    }

    fn enter_env(&mut self) {
        self.environments.push(HashMap::new());
    }
    fn exit_env(&mut self) {
        self.environments.pop();
    }

    fn set_var(&mut self, name: String, val: Value) {
        self.environments.last_mut().unwrap().insert(name, val);
    }

    fn get_var(&self, name: &str) -> Value {
        for env in self.environments.iter().rev() {
            if let Some(v) = env.get(name) {
                return v.clone();
            }
        }
        panic!("Runtime Error: Undefined variable {}", name);
    }

    fn update_var(&mut self, name: &str, val: Value) {
        for env in self.environments.iter_mut().rev() {
            if env.contains_key(name) {
                env.insert(name.to_string(), val);
                return;
            }
        }
    }

    pub fn execute(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts {
            self.exec_stmt(&stmt);
        }
    }

    fn exec_block(&mut self, body: &[Stmt]) -> Option<Flow> {
        self.enter_env();
        let mut result = None;
        for s in body {
            if let Some(f) = self.exec_stmt(s) {
                result = Some(f);
                break;
            }
        }
        self.exit_env();
        result
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Option<Flow> {
        match stmt {
            Stmt::VarDecl { name, value, .. } => {
                let val = self.eval_expr(value);
                self.set_var(name.clone(), val);
                None
            }
            Stmt::Assign { name, value } => {
                let val = self.eval_expr(value);
                self.update_var(name, val);
                None
            }
            Stmt::ArrayIndexAssign {
                array_name,
                index,
                value,
            } => {
                let idx = match self.eval_expr(index) {
                    Value::Int(i) => i as usize,
                    _ => panic!("Runtime Error: array index must be int"),
                };
                let val = self.eval_expr(value);
                let current_arr = self.get_var(array_name);
                if let Value::Array(mut arr) = current_arr {
                    arr[idx] = val;
                    self.update_var(array_name, Value::Array(arr));
                }
                None
            }
            Stmt::FunctionDecl {
                name, params, body, ..
            } => {
                let func_val = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                self.set_var(name.clone(), func_val);
                None
            }
            Stmt::If {
                cond,
                body,
                else_body,
            } => {
                if let Value::Bool(true) = self.eval_expr(cond) {
                    self.exec_block(body)
                } else if let Some(eb) = else_body {
                    self.exec_block(eb)
                } else {
                    None
                }
            }
            Stmt::ForRange {
                iter_var,
                max,
                body,
            } => {
                let max_val = match self.eval_expr(max) {
                    Value::Int(n) => n,
                    _ => panic!("Runtime Error: range() needs an int"),
                };
                for i in 0..max_val {
                    self.enter_env();
                    self.set_var(iter_var.clone(), Value::Int(i));
                    let result = self.exec_block(body);
                    self.exit_env();
                    match result {
                        Some(Flow::Break) => break,
                        Some(Flow::Continue) | None => {}
                        Some(r @ Flow::Return(_)) => return Some(r),
                    }
                }
                None
            }
            Stmt::While { cond, body } => {
                while let Value::Bool(true) = self.eval_expr(cond) {
                    match self.exec_block(body) {
                        Some(Flow::Break) => break,
                        Some(Flow::Continue) | None => {}
                        Some(r @ Flow::Return(_)) => return Some(r),
                    }
                }
                None
            }
            Stmt::Match {
                expr,
                cases,
                default_case,
            } => {
                let match_val = self.eval_expr(expr);
                for (case_expr, body) in cases {
                    if self.eval_expr(case_expr) == match_val {
                        return match self.exec_block(body) {
                            Some(Flow::Break) => None,
                            other => other,
                        };
                    }
                }
                if let Some(def) = default_case {
                    return self.exec_block(def);
                }
                None
            }
            Stmt::Print(expr) => {
                match self.eval_expr(expr) {
                    Value::Int(n) => println!("{}", n),
                    Value::Float(f) => println!("{}", f),
                    Value::String(s) => println!("{}", s),
                    Value::Bool(b) => println!("{}", b),
                    Value::Char(c) => println!("{}", c),
                    Value::Array(arr) => println!("{:?}", arr),
                    Value::Null => println!("null"),
                    _ => println!("(Unknown Object)"),
                }
                None
            }
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr);
                None
            }
            Stmt::Break => Some(Flow::Break),
            Stmt::Continue => Some(Flow::Continue),
            Stmt::Return(opt_expr) => {
                let v = match opt_expr {
                    Some(e) => self.eval_expr(e),
                    None => Value::Void,
                };
                Some(Flow::Return(v))
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::IntLit(n) => Value::Int(*n),
            Expr::FloatLit(f) => Value::Float(*f),
            Expr::StringLit(s) => Value::String(s.clone()),
            Expr::BoolLit(b) => Value::Bool(*b),
            Expr::CharLit(c) => Value::Char(*c),
            Expr::NullLit => Value::Null,
            Expr::Ident(name) => self.get_var(name),
            Expr::ArrayLit(elements) => {
                Value::Array(elements.iter().map(|e| self.eval_expr(e)).collect())
            }
            Expr::ArrayIndex { array, index } => {
                if let (Value::Array(vec), Value::Int(i)) =
                    (self.eval_expr(array), self.eval_expr(index))
                {
                    vec[i as usize].clone()
                } else {
                    panic!("Invalid array indexing");
                }
            }
            Expr::MethodCall {
                obj,
                method,
                args: _args,
            } => {
                if let Value::Array(vec) = self.eval_expr(obj) {
                    if method == "len" {
                        return Value::Int(vec.len() as i64);
                    }
                }
                panic!("Method error");
            }
            Expr::Ternary {
                cond,
                true_expr,
                false_expr,
            } => {
                if let Value::Bool(true) = self.eval_expr(cond) {
                    self.eval_expr(true_expr)
                } else {
                    self.eval_expr(false_expr)
                }
            }
            Expr::PreIncrement(name) | Expr::PreDecrement(name) => {
                let delta = if matches!(expr, Expr::PreIncrement(_)) {
                    1
                } else {
                    -1
                };
                if let Value::Int(n) = self.get_var(name) {
                    let val = Value::Int(n + delta);
                    self.update_var(name, val.clone());
                    val
                } else {
                    panic!("Increment/decrement requires int");
                }
            }
            Expr::UnaryOp(op, e) => match (op, self.eval_expr(e)) {
                (Token::Minus, Value::Int(n)) => Value::Int(-n),
                (Token::Minus, Value::Float(f)) => Value::Float(-f),
                (Token::Not, Value::Bool(b)) => Value::Bool(!b),
                _ => panic!("Runtime Error: invalid unary operation"),
            },
            Expr::BinaryOp(l, op, r) => {
                if matches!(op, Token::And | Token::Or) {
                    let lb = match self.eval_expr(l) {
                        Value::Bool(b) => b,
                        _ => panic!("Runtime Error: and/or need bool operands"),
                    };
                    if matches!(op, Token::And) && !lb {
                        return Value::Bool(false);
                    }
                    if matches!(op, Token::Or) && lb {
                        return Value::Bool(true);
                    }
                    return match self.eval_expr(r) {
                        Value::Bool(b) => Value::Bool(b),
                        _ => panic!("Runtime Error: and/or need bool operands"),
                    };
                }

                let left = self.eval_expr(l);
                let right = self.eval_expr(r);
                match (&left, op, &right) {
                    (Value::Int(a), _, Value::Int(b)) => {
                        let (a, b) = (*a, *b);
                        match op {
                            Token::Plus => {
                                Value::Int(a.checked_add(b).expect("Runtime Error: overflow"))
                            }
                            Token::Minus => {
                                Value::Int(a.checked_sub(b).expect("Runtime Error: overflow"))
                            }
                            Token::Multiply => {
                                Value::Int(a.checked_mul(b).expect("Runtime Error: overflow"))
                            }
                            Token::Divide => {
                                if b == 0 {
                                    panic!("Runtime Error: division by zero");
                                }
                                Value::Int(a / b)
                            }
                            Token::Modulo => {
                                if b == 0 {
                                    panic!("Runtime Error: modulo by zero");
                                }
                                Value::Int(a % b)
                            }
                            Token::GreaterThan => Value::Bool(a > b),
                            Token::LessThan => Value::Bool(a < b),
                            Token::GreaterEq => Value::Bool(a >= b),
                            Token::LessEq => Value::Bool(a <= b),
                            Token::Equals => Value::Bool(a == b),
                            Token::NotEquals => Value::Bool(a != b),
                            _ => panic!("Runtime Error: unsupported operator {:?}", op),
                        }
                    }
                    (Value::String(a), Token::Plus, Value::String(b)) => {
                        Value::String(format!("{}{}", a, b))
                    }
                    (Value::Char(a), _, Value::Char(b)) => match op {
                        Token::Equals => Value::Bool(a == b),
                        Token::NotEquals => Value::Bool(a != b),
                        Token::GreaterThan => Value::Bool(a > b),
                        Token::LessThan => Value::Bool(a < b),
                        Token::GreaterEq => Value::Bool(a >= b),
                        Token::LessEq => Value::Bool(a <= b),
                        _ => panic!("Runtime Error: unsupported operator {:?} on char", op),
                    },
                    _ => {
                        if let (Some(a), Some(b)) = (to_f64(&left), to_f64(&right)) {
                            match op {
                                Token::Plus => Value::Float(a + b),
                                Token::Minus => Value::Float(a - b),
                                Token::Multiply => Value::Float(a * b),
                                Token::Divide => Value::Float(a / b),
                                Token::GreaterThan => Value::Bool(a > b),
                                Token::LessThan => Value::Bool(a < b),
                                Token::GreaterEq => Value::Bool(a >= b),
                                Token::LessEq => Value::Bool(a <= b),
                                Token::Equals => Value::Bool(a == b),
                                Token::NotEquals => Value::Bool(a != b),
                                _ => {
                                    panic!("Runtime Error: unsupported operator {:?} on floats", op)
                                }
                            }
                        } else {
                            match op {
                                Token::Equals => Value::Bool(left == right),
                                Token::NotEquals => Value::Bool(left != right),
                                _ => panic!(
                                    "Runtime Error: cannot apply {:?} to {:?} and {:?}",
                                    op, left, right
                                ),
                            }
                        }
                    }
                }
            }
            Expr::FunctionCall { name, args } => {
                if name == "input" {
                    if let Some(p) = args.get(0) {
                        if let Value::String(s) = self.eval_expr(p) {
                            print!("{}", s);
                        }
                    }
                    io::stdout().flush().unwrap();
                    let mut line = String::new();
                    io::stdin().read_line(&mut line).unwrap();
                    return Value::String(line.trim_end().to_string());
                }

                if let Value::Function { params, body } = self.get_var(name) {
                    let arg_vals: Vec<Value> = args.iter().map(|a| self.eval_expr(a)).collect();
                    self.enter_env();
                    for (i, param) in params.iter().enumerate() {
                        let arg_val = if i < arg_vals.len() {
                            arg_vals[i].clone()
                        } else {
                            self.eval_expr(param.default_val.as_ref().expect("Missing argument"))
                        };
                        self.set_var(param.name.clone(), arg_val);
                    }
                    let mut ret_val = Value::Void;
                    for s in &body {
                        if let Some(Flow::Return(v)) = self.exec_stmt(s) {
                            ret_val = v;
                            break;
                        }
                    }
                    self.exit_env();
                    ret_val
                } else {
                    panic!("Not a function");
                }
            }
        }
    }
}
