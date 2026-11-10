#![allow(dead_code, unused_variables)]
use crate::ast::{Expr, Stmt, Token, Type};
use std::collections::HashMap;

#[derive(Clone)]
pub struct VarInfo {
    pub var_type: Type,
    pub is_mut: bool,
}

fn is_int_type(t: &Type) -> bool {
    matches!(
        t,
        Type::Int
            | Type::Int8
            | Type::Int32
            | Type::Int64
            | Type::Uint
            | Type::Uint8
            | Type::Uint32
            | Type::Uint64
    )
}

fn is_float_type(t: &Type) -> bool {
    matches!(t, Type::Float | Type::Float32)
}

pub struct SemanticAnalyzer {
    scopes: Vec<HashMap<String, VarInfo>>,
    loop_depth: usize,
    current_func_ret: Option<Type>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            scopes: vec![HashMap::new()],
            loop_depth: 0,
            current_func_ret: None,
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, var_type: Type, is_mut: bool) {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(name) {
            panic!("Compile Error: '{}' already declared", name);
        }
        scope.insert(name.to_string(), VarInfo { var_type, is_mut });
    }

    fn lookup(&self, name: &str) -> Option<VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info.clone());
            }
        }
        None
    }

    pub fn analyze(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                is_mut,
                explicit_type,
                value,
            } => {
                let val_type = self.eval_type(value);
                let final_type = match explicit_type {
                    Some(t) => {
                        if is_int_type(t) && is_int_type(&val_type) {
                            t.clone()
                        } else if is_float_type(t) && is_float_type(&val_type) {
                            t.clone()
                        } else if t != &val_type && val_type != Type::Null {
                            panic!("Type mismatch: Expected {:?}, got {:?}", t, val_type);
                        } else {
                            t.clone()
                        }
                    }
                    None => val_type,
                };
                self.declare(name, final_type, *is_mut);
            }
            Stmt::Assign { name, value } => {
                let var_info = self.lookup(name).expect("Undeclared variable");
                if !var_info.is_mut {
                    panic!("Cannot reassign const '{}'", name);
                }
                let val_type = self.eval_type(value);
                let compatible = var_info.var_type == val_type
                    || (is_int_type(&var_info.var_type) && is_int_type(&val_type))
                    || (is_float_type(&var_info.var_type) && is_float_type(&val_type));
                if !compatible {
                    panic!(
                        "Type mismatch on assignment to '{}': expected {:?}, got {:?}",
                        name, var_info.var_type, val_type
                    );
                }
            }
            Stmt::FunctionDecl {
                name,
                params,
                ret_type,
                body,
            } => {
                self.declare(name, ret_type.clone(), false);
                self.enter_scope();
                let prev_ret = self.current_func_ret.clone();
                self.current_func_ret = Some(ret_type.clone());

                for p in params {
                    self.declare(&p.name, p.param_type.clone(), false);
                }
                self.analyze(body);

                self.current_func_ret = prev_ret;
                self.exit_scope();
            }
            Stmt::If {
                cond,
                body,
                else_body,
            } => {
                if self.eval_type(cond) != Type::Bool {
                    panic!("If condition must be bool");
                }
                self.enter_scope();
                self.analyze(body);
                self.exit_scope();
                if let Some(eb) = else_body {
                    self.enter_scope();
                    self.analyze(eb);
                    self.exit_scope();
                }
            }
            Stmt::ForRange {
                iter_var,
                max,
                body,
            } => {
                if !is_int_type(&self.eval_type(max)) {
                    panic!("Range max must be an integer");
                }
                self.loop_depth += 1;
                self.enter_scope();
                self.declare(iter_var, Type::Int, false);
                self.analyze(body);
                self.exit_scope();
                self.loop_depth -= 1;
            }
            Stmt::While { cond, body } => {
                if self.eval_type(cond) != Type::Bool {
                    panic!("Condition must be bool");
                }
                self.loop_depth += 1;
                self.enter_scope();
                self.analyze(body);
                self.exit_scope();
                self.loop_depth -= 1;
            }
            Stmt::Match {
                expr,
                cases,
                default_case,
            } => {
                let main_type = self.eval_type(expr);
                self.loop_depth += 1;
                for (case_expr, body) in cases {
                    if self.eval_type(case_expr) != main_type {
                        panic!("Match case type mismatch");
                    }
                    self.enter_scope();
                    self.analyze(body);
                    self.exit_scope();
                }
                if let Some(def) = default_case {
                    self.enter_scope();
                    self.analyze(def);
                    self.exit_scope();
                }
                self.loop_depth -= 1;
            }
            Stmt::Break | Stmt::Continue => {
                if self.loop_depth == 0 {
                    panic!("Break/Continue outside of loop");
                }
            }
            Stmt::Return(opt_expr) => {
                let expected = self
                    .current_func_ret
                    .clone()
                    .expect("Return outside function");
                let actual = opt_expr
                    .as_ref()
                    .map(|e| self.eval_type(e))
                    .unwrap_or(Type::Void);

                let is_valid = expected == actual
                    || (is_int_type(&expected) && is_int_type(&actual))
                    || (is_float_type(&expected) && is_float_type(&actual));

                if !is_valid {
                    panic!(
                        "Return type mismatch: expected {:?}, got {:?}",
                        expected, actual
                    );
                }
            }
            Stmt::ExprStmt(expr) | Stmt::Print(expr) => {
                self.eval_type(expr);
            }
            _ => {}
        }
    }

    fn eval_type(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::IntLit(_) => Type::Int,
            Expr::FloatLit(_) => Type::Float,
            Expr::StringLit(_) => Type::String,
            Expr::BoolLit(_) => Type::Bool,
            Expr::CharLit(_) => Type::Char,
            Expr::NullLit => Type::Null,
            Expr::Ident(name) => {
                self.lookup(name)
                    .expect(&format!("Undeclared: {}", name))
                    .var_type
            }
            Expr::ArrayLit(elements) => {
                if elements.is_empty() {
                    panic!("Cannot infer type of empty array");
                }
                let inner = self.eval_type(&elements[0]);
                Type::Array(Box::new(inner))
            }
            Expr::ArrayIndex { array, index } => {
                if !is_int_type(&self.eval_type(index)) {
                    panic!("Array index must be int");
                }
                if let Type::Array(inner) = self.eval_type(array) {
                    *inner
                } else {
                    panic!("Cannot index into non-array");
                }
            }
            Expr::Ternary {
                cond,
                true_expr,
                false_expr,
            } => {
                if self.eval_type(cond) != Type::Bool {
                    panic!("Ternary cond must be bool");
                }
                let t = self.eval_type(true_expr);
                if t != self.eval_type(false_expr) {
                    panic!("Ternary branches must match");
                }
                t
            }
            Expr::UnaryOp(op, e) => {
                let e_type = self.eval_type(e);
                match op {
                    Token::Minus => e_type,
                    Token::Not => {
                        if e_type != Type::Bool {
                            panic!("'not' operator requires bool");
                        }
                        Type::Bool
                    }
                    _ => Type::Void,
                }
            }
            Expr::BinaryOp(l, token, r) => {
                let l_type = self.eval_type(l);
                let r_type = self.eval_type(r);

                let both_numeric = (is_int_type(&l_type) || is_float_type(&l_type))
                    && (is_int_type(&r_type) || is_float_type(&r_type));

                let res_type = if both_numeric {
                    if is_float_type(&l_type) || is_float_type(&r_type) {
                        Type::Float
                    } else {
                        Type::Int
                    }
                } else {
                    if l_type != r_type {
                        panic!("Type mismatch in binary op: {:?} vs {:?}", l_type, r_type);
                    }
                    l_type
                };

                match token {
                    Token::Plus
                    | Token::Minus
                    | Token::Multiply
                    | Token::Divide
                    | Token::Modulo => res_type,
                    Token::Equals
                    | Token::NotEquals
                    | Token::GreaterThan
                    | Token::LessThan
                    | Token::GreaterEq
                    | Token::LessEq => Type::Bool,
                    Token::And | Token::Or => Type::Bool,
                    _ => panic!("Unsupported binary token: {:?}", token),
                }
            }
            Expr::MethodCall {
                obj,
                method,
                args: _args,
            } => {
                if method == "len" {
                    if let Type::Array(_) = self.eval_type(obj) {
                        return Type::Int;
                    }
                    panic!(".len() only available on arrays");
                }
                panic!("Unknown method");
            }
            Expr::FunctionCall { name, args } => {
                for a in args {
                    self.eval_type(a);
                }
                if name == "input" {
                    return Type::String;
                }
                self.lookup(name)
                    .expect(&format!("Undeclared fn: {}", name))
                    .var_type
            }
            Expr::PreIncrement(name) | Expr::PreDecrement(name) => {
                let info = self.lookup(name).expect("Undeclared");
                if !info.is_mut {
                    panic!("Cannot increment const");
                }
                info.var_type
            }
        }
    }
}
