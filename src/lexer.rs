#![allow(dead_code)]
use crate::ast::Token;

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.pos += 1;
        ch
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\n' | '\r' | '\t' => {
                    self.advance();
                }
                '/' => {
                    self.advance();
                    if self.peek() == Some('/') {
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else {
                        tokens.push(Token::Divide);
                    }
                }
                ';' => {
                    tokens.push(Token::Semi);
                    self.advance();
                }
                ':' => {
                    tokens.push(Token::Colon);
                    self.advance();
                }
                '?' => {
                    tokens.push(Token::Question);
                    self.advance();
                }
                '.' => {
                    tokens.push(Token::Dot);
                    self.advance();
                }
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                }
                '{' => {
                    tokens.push(Token::OpenBrace);
                    self.advance();
                }
                '}' => {
                    tokens.push(Token::CloseBrace);
                    self.advance();
                }
                '(' => {
                    tokens.push(Token::OpenParen);
                    self.advance();
                }
                ')' => {
                    tokens.push(Token::CloseParen);
                    self.advance();
                }
                '[' => {
                    tokens.push(Token::OpenBracket);
                    self.advance();
                }
                ']' => {
                    tokens.push(Token::CloseBracket);
                    self.advance();
                }
                '+' => {
                    self.advance();
                    if self.peek() == Some('+') {
                        tokens.push(Token::PlusPlus);
                        self.advance();
                    } else {
                        tokens.push(Token::Plus);
                    }
                }
                '-' => {
                    self.advance();
                    if self.peek() == Some('-') {
                        tokens.push(Token::MinusMinus);
                        self.advance();
                    } else if self.peek() == Some('>') {
                        tokens.push(Token::Minus);
                        self.advance();
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => {
                    tokens.push(Token::Multiply);
                    self.advance();
                }
                '%' => {
                    tokens.push(Token::Modulo);
                    self.advance();
                }
                '=' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        tokens.push(Token::Equals);
                        self.advance();
                    } else {
                        tokens.push(Token::Assign);
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        tokens.push(Token::NotEquals);
                        self.advance();
                    } else {
                        tokens.push(Token::Not);
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        tokens.push(Token::GreaterEq);
                        self.advance();
                    } else {
                        tokens.push(Token::GreaterThan);
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        tokens.push(Token::LessEq);
                        self.advance();
                    } else {
                        tokens.push(Token::LessThan);
                    }
                }
                '"' => {
                    self.advance();
                    let mut s = String::new();
                    while let Some(c) = self.advance() {
                        if c == '"' {
                            break;
                        }
                        s.push(c);
                    }
                    tokens.push(Token::StringLit(s));
                }
                '\'' => {
                    self.advance();
                    let c = self.advance().expect("Expected character");
                    self.advance(); // consume closing quote
                    tokens.push(Token::CharLit(c));
                }
                c if c.is_alphabetic() || c == '_' => {
                    let mut id = String::new();
                    while let Some(c) = self.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            id.push(self.advance().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(match id.as_str() {
                        "let" => Token::Let,
                        "const" => Token::Const,
                        "if" => Token::If,
                        "elif" => Token::Elif,
                        "else" => Token::Else,
                        "for" => Token::For,
                        "in" => Token::In,
                        "range" => Token::Range,
                        "while" => Token::While,
                        "break" => Token::Break,
                        "continue" => Token::Continue,
                        "match" => Token::Match,
                        "case" => Token::Case,
                        "default" => Token::Default,
                        "fn" => Token::Fn,
                        "return" => Token::Return,
                        "null" => Token::Null,
                        "int" | "int8" | "int32" | "int64" | "uint" | "uint8" | "uint32"
                        | "uint64" => Token::IntType,
                        "string" => Token::StringType,
                        "bool" => Token::BoolType,
                        "float32" | "float64" => Token::FloatType,
                        "char" => Token::CharType,
                        "true" => Token::BoolLit(true),
                        "false" => Token::BoolLit(false),
                        "and" => Token::And,
                        "or" => Token::Or,
                        "not" => Token::Not,
                        _ => Token::Ident(id),
                    });
                }
                c if c.is_ascii_digit() => {
                    let mut num = String::new();
                    let mut is_float = false;
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            num.push(self.advance().unwrap());
                        } else if c == '.' && !is_float {
                            is_float = true;
                            num.push(self.advance().unwrap());
                        } else {
                            break;
                        }
                    }
                    if is_float {
                        tokens.push(Token::FloatLit(num.parse().unwrap()));
                    } else {
                        tokens.push(Token::IntLit(num.parse().unwrap()));
                    }
                }
                _ => {
                    panic!(
                        "Lexer Error: Unexpected character '{}'",
                        self.peek().unwrap()
                    );
                }
            }
        }

        tokens.push(Token::EOF);
        tokens
    }
}
