#![allow(dead_code, unused_variables)]
use crate::ast::{Expr, Stmt, Token};

pub struct NativeCodegen {
    output: String,
    indent_level: usize,
}

impl NativeCodegen {
    pub fn new() -> Self {
        NativeCodegen {
            output: String::from(
                "#include <stdio.h>\n#include <stdint.h>\n#include <stdbool.h>\n\nint main() {\n",
            ),
            indent_level: 1,
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
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

            // NEW: Delete the intermediate .c file upon success
            if let Err(e) = std::fs::remove_file(&c_filename) {
                eprintln!("[-] Warning: Failed to clean up {}: {}", c_filename, e);
            } else {
                println!("[-] Cleaned up intermediate file: {}", c_filename);
            }
        } else {
            // Leave the .c file on the disk so you can see what caused the C compiler to fail
            eprintln!(
                "[!] Native compilation failed. Keeping {} for debugging.",
                c_filename
            );
        }
    }
    fn gen_stmt(&mut self, stmt: &Stmt) {
        let ind = self.indent();
        match stmt {
            Stmt::VarDecl { name, value, .. } => {
                let val_str = self.gen_expr(value);
                self.output
                    .push_str(&format!("{}int64_t {} = {};\n", ind, name, val_str));
            }
            Stmt::Assign { name, value } => {
                let val_str = self.gen_expr(value);
                self.output
                    .push_str(&format!("{}{} = {};\n", ind, name, val_str));
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
                if let Expr::StringLit(s) = expr {
                    self.output
                        .push_str(&format!("{}printf(\"%s\\n\", \"{}\");\n", ind, s));
                } else {
                    let expr_str = self.gen_expr(expr);
                    self.output.push_str(&format!(
                        "{}printf(\"%lld\\n\", (long long)({}));\n",
                        ind, expr_str
                    ));
                }
            }
            _ => {}
        }
    }

    fn gen_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit(n) => format!("{}LL", n),
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
