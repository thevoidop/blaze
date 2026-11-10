#![allow(dead_code)]
use crate::ast::{Expr, Param, Stmt, Token, Type};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn consume(&mut self) -> Token {
        let t = self.peek().clone();
        self.pos += 1;
        t
    }
    fn expect(&mut self, expected: Token) {
        if self.peek() == &expected {
            self.consume();
        } else {
            panic!(
                "Parser Error: Expected {:?}, found {:?}",
                expected,
                self.peek()
            );
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while self.peek() != &Token::EOF {
            stmts.push(self.parse_stmt());
        }
        stmts
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            Token::Let | Token::Const => self.parse_var_decl(),
            Token::Fn => self.parse_function_decl(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Match => self.parse_match(),
            Token::Break => {
                self.consume();
                self.expect(Token::Semi);
                Stmt::Break
            }
            Token::Continue => {
                self.consume();
                self.expect(Token::Semi);
                Stmt::Continue
            }
            Token::Return => self.parse_return(),
            Token::Ident(_) => {
                if let Token::Ident(id) = self.peek() {
                    if id == "print" {
                        self.consume();
                        self.expect(Token::OpenParen);
                        let expr = self.parse_expression();
                        self.expect(Token::CloseParen);
                        self.expect(Token::Semi);
                        return Stmt::Print(expr);
                    }
                }

                let expr = self.parse_expression();

                match self.peek() {
                    Token::Assign => {
                        self.consume();
                        let value = self.parse_expression();
                        self.expect(Token::Semi);
                        if let Expr::Ident(name) = expr {
                            Stmt::Assign { name, value }
                        } else if let Expr::ArrayIndex { array, index } = expr {
                            if let Expr::Ident(name) = *array {
                                Stmt::ArrayIndexAssign {
                                    array_name: name,
                                    index: *index,
                                    value,
                                }
                            } else {
                                panic!("Invalid array assignment target");
                            }
                        } else {
                            panic!("Invalid assignment target");
                        }
                    }
                    Token::Semi => {
                        self.consume();
                        Stmt::ExprStmt(expr)
                    }
                    _ => panic!("Expected ';' or '=' after expression"),
                }
            }
            _ => panic!("Unexpected token: {:?}", self.peek()),
        }
    }

    fn parse_type(&mut self) -> Type {
        match self.consume() {
            Token::IntType => Type::Int,
            Token::FloatType => Type::Float,
            Token::Float32Type => Type::Float32, // <-- Added
            Token::StringType => Type::String,
            Token::BoolType => Type::Bool,
            Token::CharType => Type::Char,
            Token::OpenBracket => {
                let inner = self.parse_type();
                self.expect(Token::CloseBracket);
                Type::Array(Box::new(inner))
            }
            _ => panic!("Expected type"),
        }
    }

    fn parse_var_decl(&mut self) -> Stmt {
        let is_mut = match self.consume() {
            Token::Let => true,
            Token::Const => false,
            _ => unreachable!(),
        };

        let mut explicit_type = None;
        if matches!(
            self.peek(),
            Token::IntType
                | Token::StringType
                | Token::BoolType
                | Token::FloatType
                | Token::Float32Type
                | Token::CharType
                | Token::OpenBracket
        ) {
            explicit_type = Some(self.parse_type());
        }

        let name = match self.consume() {
            Token::Ident(id) => id,
            _ => panic!("Expected identifier"),
        };
        self.expect(Token::Assign);
        let value = self.parse_expression();
        self.expect(Token::Semi);

        Stmt::VarDecl {
            name,
            is_mut,
            explicit_type,
            value,
        }
    }

    fn parse_function_decl(&mut self) -> Stmt {
        self.consume();
        let name = match self.consume() {
            Token::Ident(id) => id,
            _ => panic!("Expected function name"),
        };
        self.expect(Token::OpenParen);

        let mut params = Vec::new();
        while self.peek() != &Token::CloseParen {
            let p_name = match self.consume() {
                Token::Ident(id) => id,
                _ => panic!("Expected parameter name"),
            };
            self.expect(Token::Colon);
            let p_type = self.parse_type();

            let mut default_val = None;
            if self.peek() == &Token::Assign {
                self.consume();
                default_val = Some(self.parse_expression());
            }

            params.push(Param {
                name: p_name,
                param_type: p_type,
                default_val,
            });
            if self.peek() == &Token::Comma {
                self.consume();
            }
        }
        self.expect(Token::CloseParen);

        let mut ret_type = Type::Void;
        if self.peek() == &Token::Minus {
            self.consume();
            if self.peek() == &Token::GreaterThan {
                self.consume();
            }
            ret_type = self.parse_type();
        }

        let body = self.parse_block();
        Stmt::FunctionDecl {
            name,
            params,
            ret_type,
            body,
        }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(Token::OpenBrace);
        let mut stmts = Vec::new();
        while self.peek() != &Token::CloseBrace {
            stmts.push(self.parse_stmt());
        }
        self.expect(Token::CloseBrace);
        stmts
    }

    fn parse_if(&mut self) -> Stmt {
        self.consume(); // consume 'if'
        let cond = self.parse_expression();
        let body = self.parse_block();

        let else_body = self.parse_else_or_elif();

        Stmt::If {
            cond,
            body,
            else_body,
        }
    }

    // Helper to recursively parse chains of 'elif' and final 'else'
    fn parse_else_or_elif(&mut self) -> Option<Vec<Stmt>> {
        if self.peek() == &Token::Elif {
            self.consume(); // consume 'elif'
            let cond = self.parse_expression();
            let body = self.parse_block();

            // Recursively check for subsequent elifs or else
            let else_body = self.parse_else_or_elif();

            Some(vec![Stmt::If {
                cond,
                body,
                else_body,
            }])
        } else if self.peek() == &Token::Else {
            self.consume(); // consume 'else'
            Some(self.parse_block())
        } else {
            None
        }
    }

    fn parse_while(&mut self) -> Stmt {
        self.consume();
        let cond = self.parse_expression();
        let body = self.parse_block();
        Stmt::While { cond, body }
    }

    fn parse_for(&mut self) -> Stmt {
        self.consume();
        let iter_var = match self.consume() {
            Token::Ident(id) => id,
            _ => panic!("Expected identifier"),
        };
        self.expect(Token::In);
        self.expect(Token::Range);
        self.expect(Token::OpenParen);
        let max = self.parse_expression();
        self.expect(Token::CloseParen);
        let body = self.parse_block();
        Stmt::ForRange {
            iter_var,
            max,
            body,
        }
    }

    fn parse_match(&mut self) -> Stmt {
        self.consume();
        let expr = self.parse_expression();
        self.expect(Token::OpenBrace);

        let mut cases = Vec::new();
        let mut default_case = None;

        while self.peek() != &Token::CloseBrace {
            if self.peek() == &Token::Case {
                self.consume();
                let case_expr = self.parse_expression();
                self.expect(Token::Colon);
                let body = self.parse_block();
                cases.push((case_expr, body));
            } else if self.peek() == &Token::Default {
                self.consume();
                self.expect(Token::Colon);
                default_case = Some(self.parse_block());
            } else {
                panic!("Expected 'case' or 'default'");
            }
        }
        self.expect(Token::CloseBrace);
        Stmt::Match {
            expr,
            cases,
            default_case,
        }
    }

    fn parse_return(&mut self) -> Stmt {
        self.consume();
        if self.peek() == &Token::Semi {
            self.consume();
            Stmt::Return(None)
        } else {
            let expr = self.parse_expression();
            self.expect(Token::Semi);
            Stmt::Return(Some(expr))
        }
    }

    pub fn parse_expression(&mut self) -> Expr {
        let mut expr = self.parse_logical();
        if self.peek() == &Token::Question {
            self.consume();
            let true_expr = self.parse_expression();
            self.expect(Token::Colon);
            let false_expr = self.parse_expression();
            expr = Expr::Ternary {
                cond: Box::new(expr),
                true_expr: Box::new(true_expr),
                false_expr: Box::new(false_expr),
            };
        }
        expr
    }

    fn parse_logical(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while matches!(self.peek(), Token::And | Token::Or) {
            let op = self.consume();
            let right = self.parse_equality();
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_comparison();
        while matches!(self.peek(), Token::Equals | Token::NotEquals) {
            let op = self.consume();
            let right = self.parse_comparison();
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_term();
        while matches!(
            self.peek(),
            Token::GreaterThan | Token::LessThan | Token::GreaterEq | Token::LessEq
        ) {
            let op = self.consume();
            let right = self.parse_term();
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = self.consume();
            let right = self.parse_factor();
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_factor(&mut self) -> Expr {
        let mut left = self.parse_unary();
        while matches!(self.peek(), Token::Multiply | Token::Divide | Token::Modulo) {
            let op = self.consume();
            let right = self.parse_unary();
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_unary(&mut self) -> Expr {
        if matches!(self.peek(), Token::Not | Token::Minus) {
            let op = self.consume();
            let right = self.parse_unary();
            Expr::UnaryOp(op, Box::new(right))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Expr {
        let mut expr = match self.consume() {
            Token::IntLit(n) => Expr::IntLit(n),
            Token::FloatLit(f) => Expr::FloatLit(f),
            Token::StringLit(s) => Expr::StringLit(s),
            Token::BoolLit(b) => Expr::BoolLit(b),
            Token::CharLit(c) => Expr::CharLit(c),
            Token::Null => Expr::NullLit,
            Token::OpenBracket => {
                let mut elements = Vec::new();
                while self.peek() != &Token::CloseBracket {
                    elements.push(self.parse_expression());
                    if self.peek() == &Token::Comma {
                        self.consume();
                    }
                }
                self.expect(Token::CloseBracket);
                Expr::ArrayLit(elements)
            }
            Token::OpenParen => {
                let inner = self.parse_expression();
                self.expect(Token::CloseParen);
                inner
            }
            Token::Ident(name) => {
                if self.peek() == &Token::PlusPlus {
                    self.consume();
                    Expr::PreIncrement(name)
                } else if self.peek() == &Token::MinusMinus {
                    self.consume();
                    Expr::PreDecrement(name)
                } else {
                    Expr::Ident(name)
                }
            }
            _ => panic!("Expected expression"),
        };

        loop {
            match self.peek() {
                Token::OpenBracket => {
                    self.consume();
                    let index = self.parse_expression();
                    self.expect(Token::CloseBracket);
                    expr = Expr::ArrayIndex {
                        array: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::OpenParen => {
                    if let Expr::Ident(ref name) = expr {
                        self.consume();
                        let mut args = Vec::new();
                        while self.peek() != &Token::CloseParen {
                            args.push(self.parse_expression());
                            if self.peek() == &Token::Comma {
                                self.consume();
                            }
                        }
                        self.expect(Token::CloseParen);
                        expr = Expr::FunctionCall {
                            name: name.clone(),
                            args,
                        };
                    } else {
                        panic!("Function call on invalid target");
                    }
                }
                Token::Dot => {
                    self.consume();
                    let method = match self.consume() {
                        Token::Ident(id) => id,
                        _ => panic!("Expected method name"),
                    };
                    self.expect(Token::OpenParen);
                    let mut args = Vec::new();
                    while self.peek() != &Token::CloseParen {
                        args.push(self.parse_expression());
                        if self.peek() == &Token::Comma {
                            self.consume();
                        }
                    }
                    self.expect(Token::CloseParen);
                    expr = Expr::MethodCall {
                        obj: Box::new(expr),
                        method,
                        args,
                    };
                }
                _ => break,
            }
        }
        expr
    }
}
