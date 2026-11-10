#![allow(dead_code, unused_variables)]
use crate::ast::{Expr, Stmt, Token, Type};
use std::collections::HashMap;

pub struct NativeCodegen {
    output: String,
    indent_level: usize,
    var_types: HashMap<String, Type>,
}

impl NativeCodegen {
    pub fn new() -> Self {
        NativeCodegen {
            output: String::from(
                "#include <stdio.h>\n#include <stdint.h>\n#include <stdbool.h>\n\nint main() {\n",
            ),
            indent_level: 1,
            var_types: HashMap::new(),
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn infer_expr_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::IntLit(_) => Type::Int,
            Expr::FloatLit(_) => Type::Float,
            Expr::BoolLit(_) => Type::Bool,
            Expr::StringLit(_) => Type::String,
            Expr::CharLit(_) => Type::Char,
            Expr::Ident(name) => self.var_types.get(name).cloned().unwrap_or(Type::Int),
            Expr::BinaryOp(l, op, r) => {
                if matches!(
                    op,
                    Token::GreaterThan
                        | Token::LessThan
                        | Token::GreaterEq
                        | Token::LessEq
                        | Token::Equals
                        | Token::NotEquals
                        | Token::And
                        | Token::Or
                ) {
                    return Type::Bool;
                }
                let lt = self.infer_expr_type(l);
                let rt = self.infer_expr_type(r);
                if matches!(lt, Type::Float | Type::Float32)
                    || matches!(rt, Type::Float | Type::Float32)
                {
                    Type::Float
                } else {
                    Type::Int
                }
            }
            _ => Type::Int,
        }
    }

    fn c_type(t: &Type) -> &'static str {
        match t {
            Type::Float => "double",
            Type::Float32 => "float",
            Type::Bool => "bool",
            Type::String => "const char*",
            Type::Char => "char",
            _ => "int64_t",
        }
    }

    pub fn generate(&mut self, stmts: &[Stmt], output_binary: &str) {
        for stmt in stmts {
            self.gen_stmt(stmt);
        }
        self.output.push_str("    return 0;\n}\n");

        let c_filename = format!("{}.c", output_binary);
        std::fs::write(&c_filename, &self.output).expect("Failed to write C source file");

        println!("[-] Transpiled to C: {}", c_filename);
        println!("[-] Compiling to native assembly with -O3 optimizations...");

        let status = std::process::Command::new("cc")
            .args(&["-O3", &c_filename, "-o", output_binary])
            .status()
            .expect("Failed to invoke 'cc'. Make sure gcc or clang is installed.");

        if status.success() {
            println!("[+] Native binary built successfully: ./{}", output_binary);

            if let Err(e) = std::fs::remove_file(&c_filename) {
                eprintln!("[-] Warning: Failed to clean up {}: {}", c_filename, e);
            } else {
                println!("[-] Cleaned up intermediate file: {}", c_filename);
            }
        } else {
            eprintln!(
                "[!] Native compilation failed. Keeping {} for debugging.",
                c_filename
            );
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        let ind = self.indent();
        match stmt {
            Stmt::VarDecl {
                name,
                explicit_type,
                value,
                ..
            } => {
                let val_str = self.gen_expr(value);
                let resolved_type = explicit_type
                    .clone()
                    .unwrap_or_else(|| self.infer_expr_type(value));
                let c_type_str = Self::c_type(&resolved_type);
                self.var_types.insert(name.clone(), resolved_type);
                self.output
                    .push_str(&format!("{}{} {} = {};\n", ind, c_type_str, name, val_str));
            }
            Stmt::Assign { name, value } => {
                let val_str = self.gen_expr(value);
                self.output
                    .push_str(&format!("{}{} = {};\n", ind, name, val_str));
            }
            Stmt::Match {
                expr,
                cases,
                default_case,
            } => {
                let expr_str = self.gen_expr(expr);
                for (i, (case_expr, body)) in cases.iter().enumerate() {
                    let case_str = self.gen_expr(case_expr);
                    let keyword = if i == 0 { "if" } else { " else if" };
                    self.output.push_str(&format!(
                        "{}{} (({}) == ({})) {{\n",
                        if i == 0 { &ind } else { "" },
                        keyword,
                        expr_str,
                        case_str
                    ));
                    self.indent_level += 1;
                    for s in body {
                        if !matches!(s, Stmt::Break) {
                            self.gen_stmt(s);
                        }
                    }
                    self.indent_level -= 1;
                    self.output.push_str(&format!("{}}}", ind));
                }
                if let Some(def) = default_case {
                    if cases.is_empty() {
                        self.output.push_str(&format!("{}{{\n", ind));
                    } else {
                        self.output.push_str(" else {\n");
                    }
                    self.indent_level += 1;
                    for s in def {
                        if !matches!(s, Stmt::Break) {
                            self.gen_stmt(s);
                        }
                    }
                    self.indent_level -= 1;
                    self.output.push_str(&format!("{}}}\n", ind));
                } else {
                    self.output.push('\n');
                }
            }
            Stmt::If {
                cond,
                body,
                else_body,
            } => {
                let cond_str = self.gen_expr(cond);
                self.output
                    .push_str(&format!("{}if ({}) {{\n", ind, cond_str));
                self.indent_level += 1;
                for s in body {
                    self.gen_stmt(s);
                }
                self.indent_level -= 1;
                if let Some(eb) = else_body {
                    self.output.push_str(&format!("{}}} else {{\n", ind));
                    self.indent_level += 1;
                    for s in eb {
                        self.gen_stmt(s);
                    }
                    self.indent_level -= 1;
                }
                self.output.push_str(&format!("{}}}\n", ind));
            }
            Stmt::While { cond, body } => {
                let cond_str = self.gen_expr(cond);
                self.output
                    .push_str(&format!("{}while ({}) {{\n", ind, cond_str));
                self.indent_level += 1;
                for s in body {
                    self.gen_stmt(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", ind));
            }
            Stmt::ExprStmt(expr) => {
                let expr_str = self.gen_expr(expr);
                self.output.push_str(&format!("{}{};\n", ind, expr_str));
            }
            Stmt::Print(expr) => {
                let expr_type = self.infer_expr_type(expr);
                let expr_str = self.gen_expr(expr);
                match expr_type {
                    Type::String => {
                        self.output
                            .push_str(&format!("{}printf(\"%s\\n\", {});\n", ind, expr_str));
                    }
                    Type::Float | Type::Float32 => {
                        self.output.push_str(&format!(
                            "{}printf(\"%g\\n\", (double)({}));\n",
                            ind, expr_str
                        ));
                    }
                    Type::Bool => {
                        self.output.push_str(&format!(
                            "{}printf(\"%s\\n\", ({}) ? \"true\" : \"false\");\n",
                            ind, expr_str
                        ));
                    }
                    Type::Char => {
                        self.output
                            .push_str(&format!("{}printf(\"%c\\n\", {});\n", ind, expr_str));
                    }
                    _ => {
                        self.output.push_str(&format!(
                            "{}printf(\"%lld\\n\", (long long)({}));\n",
                            ind, expr_str
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    fn gen_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit(n) => format!("{}LL", n),
            Expr::FloatLit(f) => {
                let s = format!("{}", f);
                if s.contains('.') {
                    s
                } else {
                    format!("{}.0", s)
                }
            }
            Expr::BoolLit(b) => format!("{}", b),
            Expr::StringLit(s) => format!("\"{}\"", s),
            Expr::CharLit(c) => format!("'{}'", c),
            Expr::Ident(name) => name.clone(),
            Expr::PreIncrement(name) => format!("(++{})", name),
            Expr::PreDecrement(name) => format!("(--{})", name),
            Expr::BinaryOp(left, op, right) => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                let op_str = match op {
                    Token::Plus => "+",
                    Token::Minus => "-",
                    Token::Multiply => "*",
                    Token::Divide => "/",
                    Token::Modulo => "%",
                    Token::GreaterThan => ">",
                    Token::LessThan => "<",
                    Token::GreaterEq => ">=",
                    Token::LessEq => "<=",
                    Token::Equals => "==",
                    Token::NotEquals => "!=",
                    Token::And => "&&",
                    Token::Or => "||",
                    _ => "?",
                };
                format!("({} {} {})", l, op_str, r)
            }
            _ => "0".to_string(),
        }
    }
}
