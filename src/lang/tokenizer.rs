use std::collections::HashMap;

use super::{
    panic::PanicHandler,
    types::NyxAnalyzeResult,
    utils::{is_alpha, is_digit},
};

pub struct NyxTokenizer<'a> {
    source_code: &'a str,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<&'a str, TokenType>,
}

impl<'a> NyxTokenizer<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self {
            source_code,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords: keywords(),
        }
    }

    pub fn analyze(&mut self) -> NyxAnalyzeResult {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan();
        }

        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            literal: None,
            line: self.line,
            column: self.current,
        });

        Ok(&self.tokens)
    }

    fn scan(&mut self) {
        match self.advance() {
            b'[' => self.make(TokenType::LeftBracket, None),
            b']' => self.make(TokenType::RightBracket, None),
            b'(' => self.make(TokenType::LeftParen, None),
            b')' => self.make(TokenType::RightParen, None),
            b'{' => self.make(TokenType::LeftBrace, None),
            b'}' => self.make(TokenType::RightBrace, None),
            b',' => self.make(TokenType::Comma, None),
            b'.' => self.make(TokenType::Dot, None),
            b'-' => {
                let tk: TokenType = if self.char_match(b'-') {
                    TokenType::MinusMinus
                } else {
                    TokenType::Minus
                };
                self.make(tk, None);
            }
            b'+' => {
                let tk: TokenType = if self.char_match(b'+') {
                    TokenType::PlusPlus
                } else {
                    TokenType::Plus
                };
                self.make(tk, None);
            }
            b'%' => self.make(TokenType::Arith, None),
            b';' => self.make(TokenType::Semicolon, None),
            b'*' => self.make(TokenType::Star, None),
            b':' => {
                let tk: TokenType = if self.char_match(b':') {
                    TokenType::ColonColon
                } else {
                    PanicHandler::new(
                        Some(self.line),
                        Some(self.current),
                        Some(self.source_error()),
                        "Expected other ':'.",
                    )
                    .panic();
                    TokenType::Null
                };

                self.make(tk, None);
            }
            b'!' => {
                let tk: TokenType = if self.char_match(b'=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.make(tk, None);
            }
            b'=' => {
                let tk: TokenType = if self.char_match(b'=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };

                self.make(tk, None);
            }
            b'<' => {
                let tk: TokenType = if self.char_match(b'=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };

                self.make(tk, None);
            }
            b'>' => {
                let tk: TokenType = if self.char_match(b'=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };

                self.make(tk, None);
            }
            b'/' => {
                if self.char_match(b'/') {
                    loop {
                        if self.peek() == b'\n' || self.is_at_end() {
                            break;
                        }
                        self.advance();
                    }
                } else if self.char_match(b'*') {
                    loop {
                        if self.is_at_end() || self.char_match(b'*') && self.char_match(b'/') {
                            break;
                        }
                        self.advance();

                        if self.is_at_end() && self.previous() != b'*' || self.previous() == b'/' {
                            PanicHandler::new(
                                Some(self.line),
                                Some(self.current),
                                Some(self.source_error()),
                                "Incomplete multiline comment.",
                            )
                            .panic();
                        }
                    }
                } else {
                    self.make(TokenType::Slash, None);
                }
            }
            b'|' => {
                if self.char_match(b'|') {
                    return self.make(TokenType::Or, None);
                }

                PanicHandler::new(
                    Some(self.line),
                    Some(self.current),
                    Some(self.source_error()),
                    "Expected other '|'.",
                )
                .panic();
            }

            b'&' => {
                if self.char_match(b'&') {
                    return self.make(TokenType::And, None);
                }

                PanicHandler::new(
                    Some(self.line),
                    Some(self.current),
                    Some(self.source_error()),
                    "Expected other '&'.",
                )
                .panic();
            }
            b' ' | b'\r' | b'\t' => {}
            b'\n' => self.line += 1,
            b'"' => self.string(),
            c => {
                if is_digit(c) {
                    return self.number();
                } else if is_alpha(c) {
                    return self.identifier();
                }
                PanicHandler::new(
                    Some(self.line),
                    Some(self.current),
                    Some(self.source_error()),
                    "Strange char.",
                )
                .panic();
            }
        }
    }

    fn identifier(&mut self) {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }

        if let Some(&tk_type) = self.keywords.get(self.lexeme()) {
            self.make(tk_type, None);
            return;
        }

        self.make(TokenType::Identifier, None);
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }
        if self.peek() == b'.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        match self.lexeme().parse::<f64>() {
            Ok(v) => self.make(TokenType::Number, Some(LiteralValue::FValue(v))),
            Err(_) => {
                PanicHandler::new(
                    Some(self.line),
                    Some(self.current),
                    Some(self.source_error()),
                    "Could not is to correct number.",
                )
                .panic();
            }
        }
    }

    fn peek_next(&mut self) -> u8 {
        if self.current + 1 >= self.source_code.len() {
            return b'\0';
        }

        self.source_code.chars().nth(self.current + 1).unwrap() as u8
    }

    fn string(&mut self) {
        while self.peek() != b'"' && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            PanicHandler::new(
                Some(self.line),
                Some(self.current),
                Some(self.source_error()),
                "Incomplete string.",
            )
            .panic();
        }

        self.advance();

        let v: String = String::from_utf8(
            self.source_code.as_bytes()[self.start + 1..self.current - 1].to_vec(),
        )
        .unwrap_or_else(|_| {
            PanicHandler::new(
                Some(self.line),
                Some(self.current),
                Some(self.source_error()),
                "Unrecognized character of Unicode Code Point.",
            )
            .panic();

            String::new()
        });

        self.make(TokenType::StringLit, Some(LiteralValue::SValue(v)));
    }

    fn peek(&mut self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }

        self.source_code.as_bytes()[self.current]
    }

    fn char_match(&mut self, ch: u8) -> bool {
        if !self.is_at_end() && self.source_code.as_bytes()[self.current] == ch {
            self.current += 1;
            return true;
        }

        false
    }

    fn source_error(&mut self) -> &'a str {
        if self.is_at_end() {
            return &self.source_code[self.current..];
        }

        let mut adv: isize = -1;

        while ![b'{', b'}', b'\n'].contains(&self.peek()) && !self.is_at_end() {
            self.advance();
            adv += 1;
        }

        &self.source_code[self.start - adv as usize..self.current]
    }

    fn previous(&self) -> u8 {
        self.source_code.as_bytes()[self.current - 1]
    }

    fn lexeme(&self) -> &'a str {
        &self.source_code[self.start..self.current]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source_code.len()
    }

    fn advance(&mut self) -> u8 {
        let c: u8 = self.source_code.as_bytes()[self.current];
        self.current += 1;

        c
    }

    fn make(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        self.tokens.push(Token {
            token_type,
            lexeme: self.lexeme().to_string(),
            literal,
            line: self.line,
            column: self.current,
        });
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    ColonColon,
    RightBracket,
    LeftBracket,
    Arith,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    PlusPlus,
    MinusMinus,

    Identifier,
    StringLit,
    Number,

    And,
    Clazz,
    Else,
    False,
    Fc,
    For,
    ForEach,
    In,
    Continue,
    Break,
    If,
    Elif,
    Null,
    Or,
    Write,
    Return,
    Super,
    This,
    True,
    Let,
    Const,
    While,
    Extends,
    Std,
    Lib,

    Eof,
}

fn keywords<'a>() -> HashMap<&'a str, TokenType> {
    HashMap::from([
        ("foreach", TokenType::ForEach),
        ("in", TokenType::In),
        ("and", TokenType::And),
        ("clazz", TokenType::Clazz),
        ("else", TokenType::Else),
        ("for", TokenType::For),
        ("fc", TokenType::Fc),
        ("if", TokenType::If),
        ("elif", TokenType::Elif),
        ("null", TokenType::Null),
        ("or", TokenType::Or),
        ("write", TokenType::Write),
        ("return", TokenType::Return),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("true", TokenType::True),
        ("false", TokenType::False),
        ("let", TokenType::Let),
        ("const", TokenType::Const),
        ("while", TokenType::While),
        ("std", TokenType::Std),
        ("extends", TokenType::Extends),
        ("lib", TokenType::Lib),
        ("continue", TokenType::Continue),
        ("break", TokenType::Break),
    ])
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    FValue(f64),
    SValue(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line: usize,
    pub column: usize,
}
