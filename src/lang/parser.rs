use std::{collections::HashMap, rc::Rc};

use colored::*;

use super::{
    expr::{Expr, Expr::*, LiteralValue},
    panic::PanicHandler,
    stmt::Stmt,
    tokenizer::{Token, TokenType, TokenType::*},
    types::{NyxInternalParserResult, NyxParserResult},
};

pub struct NyxParser<'a> {
    tokens: &'a Vec<Token>,
    stmts: Vec<Stmt>,
    errors: Vec<String>,
    current: usize,
    loop_nesting: u16,
    return_nesting: u16,
    id: usize,
}

impl<'a> NyxParser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens,
            stmts: Vec::new(),
            errors: Vec::new(),
            current: 0,
            loop_nesting: 0,
            return_nesting: 0,
            id: 0,
        }
    }

    pub fn parse(&mut self) -> NyxParserResult {
        while !self.is_at_end() {
            match self.declaration() {
                Ok(s) => self.stmts.push(s),
                Err(msg) => {
                    self.errors.push(msg);
                    self.synchronize();
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors.join("\n"));
        }

        Ok(&self.stmts)
    }

    fn declaration(&mut self) -> NyxInternalParserResult {
        if self.match_token(Let) {
            return self.let_declaration();
        } else if self.match_token(Const) {
            return self.const_declaration();
        } else if self.match_token(Fc) {
            return self.function();
        } else if self.match_token(Clazz) {
            return self.class_declaration();
        } else if self.match_tokens(&[Lib, Std]) {
            return self.std_declaration();
        }

        self.statement()
    }

    fn std_declaration(&mut self) -> NyxInternalParserResult {
        if self.previous().token_type != Lib {
            return Err(format!(
                "Expected 'lib' keyword before 'std'. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ));
        }

        self.consume(
            Std,
            format!(
                "Expected 'std' keyword after 'lib'. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        self.consume(
            ColonColon,
            format!(
                "Expected '::' after 'std' keyword. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        let module: Token = self.consume(
            Identifier,
            format!(
                "Expected module name after '::'. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        match self.consume(
            Semicolon,
            format!(
                "Expected ';' after module name. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        ) {
            Ok(_) => {
                if self.std_md().get(module.lexeme.as_str()).is_some() {
                    return Ok(Stmt::Std {
                        module: module.lexeme,
                        fc: None,
                    });
                }

                Err(format!(
                    "Unknown standard module. ({}:{})",
                    self.previous().line,
                    self.previous().column
                ))
            }
            Err(_) => {
                match self.consume(
                    LeftBracket,
                    format!(
                        "Expected '[' after module name. ({}:{})",
                        self.tokens[self.current].line, self.tokens[self.current].column
                    ),
                ) {
                    Ok(_) => {
                        let mut functions: Vec<String> = Vec::new();

                        while !self.check(RightBracket) {
                            if self.check(Comma) {
                                self.advance();
                                functions.push(
                                    self.consume(
                                        Identifier,
                                        format!(
                                            "Expected function name. ({}:{})",
                                            self.tokens[self.current].line,
                                            self.tokens[self.current].column
                                        ),
                                    )?
                                    .lexeme,
                                );
                                continue;
                            }

                            functions.push(
                                self.consume(
                                    Identifier,
                                    format!(
                                        "Expected function name. ({}:{})",
                                        self.tokens[self.current].line,
                                        self.tokens[self.current].column
                                    ),
                                )?
                                .lexeme,
                            );
                        }
                        if functions.is_empty() {
                            return Err(format!(
                                "Expected function names after in module. ({}:{})",
                                self.tokens[self.current].line, self.tokens[self.current].column
                            ));
                        }

                        self.consume(
                            RightBracket,
                            format!(
                                "Expected ']' after module name. ({}:{})",
                                self.tokens[self.current].line, self.tokens[self.current].column
                            ),
                        )?;

                        self.consume(
                            Semicolon,
                            format!(
                                "Expected ';' after functions names. ({}:{})",
                                self.tokens[self.current].line, self.tokens[self.current].column
                            ),
                        )?;

                        if self.std_md().get(module.lexeme.as_str()).is_some() {
                            return Ok(Stmt::Std {
                                module: module.lexeme,
                                fc: Some(functions),
                            });
                        }

                        Err(format!(
                            "[{}] Unknown standard module. ({}:{})",
                            "ERROR".bold().red(),
                            self.previous().line,
                            self.previous().column
                        ))
                    }
                    Err(_) => {
                        self.consume(
                            ColonColon,
                            format!(
                                "Expected '::' after module name. ({}:{})",
                                self.tokens[self.current].line, self.tokens[self.current].column
                            ),
                        )?;
                        let func: Token = self.consume(
                            Identifier,
                            format!(
                                "Expected function name. ({}:{})",
                                self.tokens[self.current].line, self.tokens[self.current].column
                            ),
                        )?;
                        self.consume(
                            Semicolon,
                            format!(
                                "Expected ';' after function name. ({}:{})",
                                self.tokens[self.current].line, self.tokens[self.current].column
                            ),
                        )?;

                        if let Some(valid_module) = self.std_md().get(module.lexeme.as_str()) {
                            if valid_module.contains(&func.lexeme.as_str()) {
                                return Ok(Stmt::Std {
                                    module: module.lexeme,
                                    fc: Some(vec![func.lexeme]),
                                });
                            }

                            return Err(format!(
                                "Unknown function or constant in standard library. ({}:{})",
                                self.previous().line,
                                self.previous().column
                            ));
                        }

                        Err(format!(
                            "[{}] Unknown standard module. ({}:{})",
                            "ERROR".bold().red(),
                            self.previous().line,
                            self.previous().column
                        ))
                    }
                }
            }
        }
    }

    fn class_declaration(&mut self) -> NyxInternalParserResult {
        let name: Token = self.consume(
            Identifier,
            format!(
                "Expected class name. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;
        let superclass: Option<Expr> = if self.match_token(TokenType::Less) {
            self.consume(
                Identifier,
                format!(
                    "Expected superclass name. ({}:{})",
                    self.tokens[self.current].line, self.tokens[self.current].column
                ),
            )?;
            Some(Expr::Variable {
                id: self.get_id(),
                name: self.previous(),
            })
        } else {
            None
        };

        self.consume(
            LeftBrace,
            format!(
                "Expected LeftBrace before class body. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        let mut methods: Vec<Stmt> = vec![];

        while !self.check(RightBrace) && !self.is_at_end() {
            methods.push(self.function()?);
        }

        self.consume(
            RightBrace,
            format!(
                "Expected RightBrace after class body. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        Ok(Stmt::Clazz {
            name,
            methods,
            superclass,
        })
    }

    fn function(&mut self) -> NyxInternalParserResult {
        let name: Token = self.consume(
            Identifier,
            format!(
                "Expected name. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        self.consume(
            LeftParen,
            format!(
                "Expected '(' after function name. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        let mut parameters = vec![];
        if !self.check(RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(format!(
                        "(Function -> '{}') Cant have more than 255 arguments. ({}:{})",
                        name.lexeme,
                        self.tokens[self.current].line,
                        self.tokens[self.current].column
                    ));
                }

                let param: Token = self.consume(
                    Identifier,
                    format!(
                        "Expected parameter name. ({}:{})",
                        self.tokens[self.current].line, self.tokens[self.current].column
                    ),
                )?;
                parameters.push(param);

                if !self.match_token(Comma) {
                    break;
                }
            }
        }
        self.consume(
            RightParen,
            format!(
                "Expected ')' after parameters. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        self.consume(
            LeftBrace,
            format!(
                "Expected LeftBrace before function body. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        self.return_nesting += 1;

        let body: Vec<Stmt> = match self.block_statement()? {
            Stmt::Block { statements } => statements,
            _ => panic!("Block statement parsed something that was not a block"),
        };

        self.return_nesting -= 1;

        Ok(Stmt::Function {
            name,
            params: parameters,
            body,
        })
    }

    fn foreach_statement(&mut self) -> NyxInternalParserResult {
        let var: Token = self.peek();

        self.advance();

        self.consume(
            In,
            format!(
                "Expected 'In' keyword after variable. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        let value: Token = self.consume(
            Identifier,
            format!(
                "Expected variable after 'In' keyword. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        self.advance();

        self.loop_nesting += 1;

        let body: Stmt = self.block_statement()?;

        self.loop_nesting -= 1;

        Ok(Stmt::Iteration {
            var,
            value,
            body: Rc::new(body),
        })
    }

    fn const_declaration(&mut self) -> NyxInternalParserResult {
        let mut name: Token = self.consume(
            Identifier,
            format!(
                "Expected variable name. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        name.lexeme = format!("__const__{}", name.lexeme);

        let init: Expr = if self.match_token(Equal) {
            self.expression()?
        } else {
            return Err(format!(
                "Expected '=' after constant name. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ));
        };

        self.consume(
            Semicolon,
            format!(
                "Expected ';' after variable declaration. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        Ok(Stmt::Const { name, init })
    }

    fn let_declaration(&mut self) -> NyxInternalParserResult {
        let name: Token = self.consume(
            Identifier,
            format!(
                "Expected variable name. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        let init: Expr = if self.match_token(Equal) {
            self.expression()?
        } else {
            Literal {
                id: self.get_id(),
                value: LiteralValue::Null,
            }
        };

        self.consume(
            Semicolon,
            format!(
                "Expected ';' after variable declaration. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        Ok(Stmt::Let { name, init })
    }

    fn statement(&mut self) -> NyxInternalParserResult {
        if self.match_token(Write) {
            return self.write_statement();
        } else if self.match_token(LeftBrace) {
            return self.block_statement();
        } else if self.match_token(If) {
            return self.if_statement();
        } else if self.match_token(Elif) {
            return self.elif_statement();
        } else if self.match_token(While) {
            return self.while_statement();
        } else if self.match_token(For) {
            return self.for_statement();
        } else if self.match_token(ForEach) {
            return self.foreach_statement();
        } else if self.match_token(Return) {
            return self.return_statement();
        } else if self.match_token(Break) {
            return self.break_statement();
        } else if self.match_token(Continue) {
            return self.continue_statement();
        }

        self.expression_statement()
    }

    fn break_statement(&mut self) -> NyxInternalParserResult {
        if self.loop_nesting == 0 {
            PanicHandler::new(
                Some(self.tokens[self.current].line),
                Some(self.tokens[self.current].column),
                Some(&self.tokens[self.current].lexeme),
                "'break' disallowed outside of loop.",
            )
            .panic();
        }
        self.consume(
            Semicolon,
            format!(
                "Expect ';' after 'break'. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        Ok(Stmt::Break {
            keyword: self.peek(),
        })
    }

    fn continue_statement(&mut self) -> NyxInternalParserResult {
        if self.loop_nesting == 0 {
            PanicHandler::new(
                Some(self.tokens[self.current].line),
                Some(self.tokens[self.current].column),
                Some(&self.tokens[self.current].lexeme),
                "continue outside of loop.",
            )
            .panic();
        }
        self.consume(
            Semicolon,
            format!(
                "Expect ';' after 'continue'. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        Ok(Stmt::Continue {
            keyword: self.peek(),
        })
    }

    fn return_statement(&mut self) -> NyxInternalParserResult {
        if self.return_nesting == 0 {
            PanicHandler::new(
                Some(self.tokens[self.current].line),
                Some(self.tokens[self.current].column),
                Some(&self.tokens[self.current].lexeme),
                "'return' disallowed outside of function.",
            )
            .panic();
        }

        let keyword: Token = self.previous();
        let value: Option<Expr> = if !self.check(Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            Semicolon,
            format!(
                "Expected ';' after return value. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        Ok(Stmt::Return { keyword, value })
    }

    fn for_statement(&mut self) -> NyxInternalParserResult {
        self.consume(
            LeftParen,
            format!(
                "Expected '(' after 'for'. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        let initializer: Option<Stmt> = if self.match_token(Semicolon) {
            None
        } else if self.match_token(Let) {
            Some(self.let_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition: Option<Expr> = if !self.check(Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            Semicolon,
            format!(
                "Expected ';' after loop condition. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        let increment: Option<Expr> = if !self.check(RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            RightParen,
            format!(
                "Expected ')' after for clauses. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        self.loop_nesting += 1;

        let mut body: Stmt = self.statement()?;

        self.loop_nesting -= 1;

        if let Some(incr) = increment {
            body = Stmt::Block {
                statements: vec![body, Stmt::Expression { expr: incr }],
            };
        }

        let cond: Expr = match condition {
            Some(expr) => expr,
            None => Expr::Literal {
                id: self.get_id(),
                value: LiteralValue::True,
            },
        };

        body = Stmt::While {
            condition: cond,
            body: Rc::new(body),
        };

        if let Some(init) = initializer {
            body = Stmt::Block {
                statements: vec![init, body],
            };
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> NyxInternalParserResult {
        self.consume(
            LeftParen,
            format!(
                "Expected '(' after 'while'. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;
        let condition: Expr = self.expression()?;
        self.consume(
            RightParen,
            format!(
                "Expected ')' after while - condition. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        self.loop_nesting += 1;

        let body: Stmt = self.statement()?;

        self.loop_nesting -= 1;

        Ok(Stmt::While {
            condition,
            body: Rc::new(body),
        })
    }

    fn elif_statement(&mut self) -> NyxInternalParserResult {
        self.consume(
            LeftParen,
            format!(
                "Expected '(' after 'elif'. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;
        let predicate: Expr = self.expression()?;
        self.consume(
            RightParen,
            format!(
                "Expected ')' after elif - predicate. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        let then: Rc<Stmt> = Rc::new(self.statement()?);

        Ok(Stmt::Elif { predicate, then })
    }

    fn if_statement(&mut self) -> NyxInternalParserResult {
        self.consume(
            LeftParen,
            format!(
                "Expected '(' after 'if'. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;
        let predicate: Expr = self.expression()?;
        self.consume(
            RightParen,
            format!(
                "Expected ')' after if - predicate. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        let then: Rc<Stmt> = Rc::new(self.statement()?);
        let elf: Option<Rc<Stmt>> = if self.check(Elif) {
            Some(Rc::new(self.statement()?))
        } else {
            None
        };
        let els: Option<Rc<Stmt>> = if self.match_token(Else) {
            Some(Rc::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            predicate,
            then,
            elf,
            els,
        })
    }

    fn block_statement(&mut self) -> NyxInternalParserResult {
        let mut statements: Vec<Stmt> = vec![];

        while !self.check(RightBrace) && !self.is_at_end() {
            let decl: Stmt = self.declaration()?;
            statements.push(decl);
        }

        self.consume(
            RightBrace,
            format!(
                "Expected RightBrace after block. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        Ok(Stmt::Block { statements })
    }

    fn write_statement(&mut self) -> NyxInternalParserResult {
        let mut exprs: Vec<Expr> = Vec::new();

        exprs.push(self.expression()?);

        while self.match_token(Comma) {
            exprs.push(self.expression()?);
        }

        self.consume(
            Semicolon,
            format!(
                "Expected ';' after values. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        Ok(Stmt::Write { exprs })
    }

    fn expression_statement(&mut self) -> NyxInternalParserResult {
        let expr: Expr = self.expression()?;
        self.consume(
            Semicolon,
            format!(
                "Expected ';' after expression. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;
        Ok(Stmt::Expression { expr })
    }

    fn function_expression(&mut self) -> Result<Expr, String> {
        let paren: Token = self.consume(
            LeftParen,
            format!(
                "Expected '(' after function name. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;
        let mut parameters: Vec<Token> = vec![];
        if !self.check(RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(format!(
                        "Cant have more than 255 arguments. ({}:{})",
                        self.tokens[self.current].line, self.tokens[self.current].column
                    ));
                }

                let param: Token = self.consume(
                    Identifier,
                    format!(
                        "Expected parameter name. ({}:{})",
                        self.tokens[self.current].line, self.tokens[self.current].column
                    ),
                )?;
                parameters.push(param);

                if !self.match_token(Comma) {
                    break;
                }
            }
        }
        self.consume(
            RightParen,
            format!(
                "Expected ')' after parameters. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        self.consume(
            LeftBrace,
            format!(
                "Expected LeftBrace before anonymous function body. ({}:{})",
                self.tokens[self.current].line, self.tokens[self.current].column
            ),
        )?;

        let body: Vec<Stmt> = match self.block_statement()? {
            Stmt::Block { statements } => statements,
            _ => panic!("Block statement parsed something that was not a block."),
        };

        Ok(Expr::AnonFunction {
            id: self.get_id(),
            paren,
            arguments: parameters,
            body,
        })
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr: Expr = self.or()?;

        if self.match_token(Equal) {
            let value: Expr = self.expression()?;

            match expr {
                Variable { id: _, name } => Ok(Assign {
                    id: self.get_id(),
                    name,
                    value: Rc::from(value),
                }),
                Get {
                    id: _,
                    object,
                    name,
                } => Ok(Set {
                    id: self.get_id(),
                    object,
                    name,
                    value: Rc::new(value),
                }),
                _ => Err(format!(
                    "({}) Invalid assignment. ({}:{})",
                    self.tokens[self.current].lexeme,
                    self.tokens[self.current].line,
                    self.tokens[self.current].column
                )),
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.and()?;

        while self.match_token(Or) {
            let operator: Token = self.previous();
            let right: Expr = self.and()?;

            expr = Logical {
                id: self.get_id(),
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.equality()?;

        while self.match_token(And) {
            let operator: Token = self.previous();
            let right: Expr = self.equality()?;
            expr = Logical {
                id: self.get_id(),
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.comparison()?;
        while self.match_tokens(&[BangEqual, EqualEqual]) {
            let operator: Token = self.previous();
            let rhs: Expr = self.comparison()?;
            expr = Binary {
                id: self.get_id(),
                left: Rc::from(expr),
                operator,
                right: Rc::from(rhs),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.term()?;

        while self.match_tokens(&[Greater, GreaterEqual, Less, LessEqual]) {
            let op: Token = self.previous();
            let rhs: Expr = self.term()?;
            expr = Binary {
                id: self.get_id(),
                left: Rc::from(expr),
                operator: op,
                right: Rc::from(rhs),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.factor()?;

        while self.match_tokens(&[Minus, Plus]) {
            let op: Token = self.previous();
            let rhs: Expr = self.factor()?;
            expr = Binary {
                id: self.get_id(),
                left: Rc::from(expr),
                operator: op,
                right: Rc::from(rhs),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.unary()?;
        while self.match_tokens(&[Slash, Star]) {
            let op: Token = self.previous();
            let rhs: Expr = self.unary()?;
            expr = Binary {
                id: self.get_id(),
                left: Rc::from(expr),
                operator: op,
                right: Rc::from(rhs),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[Bang, Minus]) {
            let op: Token = self.previous();
            let rhs: Expr = self.unary()?;
            return Ok(Unary {
                id: self.get_id(),
                operator: op,
                right: Rc::from(rhs),
            });
        } else if self.match_next_tokens(&[PlusPlus, MinusMinus]) {
            let op: Token = self.advance();

            let tk_type: TokenType = match self.advance().token_type {
                TokenType::PlusPlus => TokenType::Plus,
                TokenType::MinusMinus => TokenType::Minus,
                _ => {
                    return Err(format!(
                        "Expected '++' or '--'. ({}:{})",
                        self.tokens[self.current].line, self.tokens[self.current].column
                    ))
                }
            };

            return Ok(Assign {
                id: self.get_id(),
                name: op.clone(),
                value: Rc::from(Binary {
                    id: self.get_id(),
                    left: Rc::new(Expr::Variable {
                        id: self.get_id(),
                        name: op.clone(),
                    }),
                    operator: Token {
                        token_type: tk_type,
                        lexeme: "".to_string(),
                        literal: None,
                        line: 0,
                        column: 0,
                    },
                    right: Rc::new(Expr::Literal {
                        id: self.get_id(),
                        value: LiteralValue::Number(1.0),
                    }),
                }),
            });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr: Expr = self.primary()?;

        loop {
            if self.match_token(LeftParen) {
                expr = self.finish_call(expr, None)?;
            } else if self.match_token(Dot) {
                let name: Token = self.consume(
                    Identifier,
                    format!(
                        "Expected dot. ({}:{})",
                        self.tokens[self.current].line, self.tokens[self.current].column
                    ),
                )?;
                expr = Get {
                    id: self.get_id(),
                    object: Rc::new(expr),
                    name,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, call: Expr, module: Option<String>) -> Result<Expr, String> {
        let mut arguments: Vec<Expr> = vec![];

        if !self.check(RightParen) {
            loop {
                let arg: Expr = self.expression()?;
                arguments.push(arg);
                if arguments.len() >= 255 {
                    return Err(format!(
                        "Cant have more than 255 arguments. ({}:{})",
                        self.tokens[self.current].line, self.tokens[self.current].column
                    ));
                }

                if !self.match_token(Comma) {
                    break;
                }
            }
        }
        let paren = self.consume(
            RightParen,
            format!(
                "({}) Expected ')' after arguments. ({}:{})",
                self.tokens[self.current].lexeme,
                self.tokens[self.current].line,
                self.tokens[self.current].column
            ),
        )?;

        match module {
            Some(module) => Ok(Call {
                id: self.get_id(),
                module: Some(module),
                call: Rc::new(call),
                paren,
                arguments,
            }),

            None => Ok(Call {
                id: self.get_id(),
                module: None,
                call: Rc::new(call),
                paren,
                arguments,
            }),
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let tk: Token = self.peek();

        let rs: Expr = match tk.token_type {
            LeftParen => {
                self.advance();
                let expr: Expr = self.expression()?;
                self.consume(
                    RightParen,
                    format!(
                        "({}) Expected ')' after expression. ({}:{})",
                        tk.lexeme, self.tokens[self.current].line, self.tokens[self.current].column
                    ),
                )?;
                Grouping {
                    id: self.get_id(),
                    expression: Rc::from(expr),
                }
            }

            LeftBracket => {
                self.advance();
                self.consume(
                    RightBracket,
                    format!(
                        "Expected ']' after list. ({}:{})",
                        self.tokens[self.current].line, self.tokens[self.current].column
                    ),
                )?;

                Expr::Literal {
                    id: self.get_id(),
                    value: LiteralValue::List(Vec::new()),
                }
            }

            False | True | Null | Number | StringLit => {
                self.advance();
                Literal {
                    id: self.get_id(),
                    value: LiteralValue::from_token(tk),
                }
            }
            Identifier => {
                self.advance();

                if self.check(TokenType::ColonColon) {
                    let module: Token = self.previous();

                    self.advance();

                    let name: Token = self.consume(
                        TokenType::Identifier,
                        format!(
                            "({}) Expected function or constant after '::'. ({}:{})",
                            tk.lexeme,
                            self.tokens[self.current].line,
                            self.tokens[self.current].column
                        ),
                    )?;

                    if self.peek().token_type == TokenType::Semicolon {
                        return Ok(Expr::ModuleProperty {
                            id: self.get_id(),
                            module: module.lexeme.to_string(),
                            name,
                        });
                    } else if self.advance().token_type == TokenType::LeftParen {
                        let new_id: usize = self.get_id();

                        let call: Expr = self.finish_call(
                            Expr::Literal {
                                id: new_id,
                                value: LiteralValue::StringValue(name.lexeme.to_string()),
                            },
                            Some(module.lexeme),
                        )?;

                        return Ok(call);
                    }
                }

                Variable {
                    id: self.get_id(),
                    name: self.previous(),
                }
            }
            TokenType::This => {
                self.advance();
                Expr::This {
                    id: self.get_id(),
                    keyword: tk,
                }
            }
            TokenType::Super => {
                self.advance();
                self.consume(
                    TokenType::Dot,
                    format!(
                        "({}) Expected '.' after 'super'. ({}:{})",
                        tk.lexeme, self.tokens[self.current].line, self.tokens[self.current].column
                    ),
                )?;
                let method: Token = self.consume(
                    TokenType::Identifier,
                    format!(
                        "({}) Expected method name. ({}:{})",
                        tk.lexeme, self.tokens[self.current].line, self.tokens[self.current].column
                    ),
                )?;
                Expr::Super {
                    id: self.get_id(),
                    keyword: tk,
                    method,
                }
            }
            Fc => {
                self.advance();
                self.function_expression()?
            }

            _ => {
                return Err(format!(
                    "\nExpected correctly syntax in this code block. ({}:{})",
                    tk.line, tk.column
                ));
            }
        };

        Ok(rs)
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn consume(&mut self, token_type: TokenType, msg: String) -> Result<Token, String> {
        let token: Token = self.peek();
        if token.token_type == token_type {
            self.advance();
            let token: Token = self.previous();
            return Ok(token);
        }

        Err(msg)
    }

    fn peek_next(&mut self) -> &Token {
        &self.tokens[self.current + 1]
    }

    fn check_next(&mut self, typ: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek_next().token_type == typ
    }

    fn match_next_tokens(&mut self, tk: &[TokenType]) -> bool {
        for tt in tk {
            if self.check_next(*tt) {
                return true;
            }
        }

        false
    }

    fn check(&mut self, typ: TokenType) -> bool {
        self.peek().token_type == typ
    }

    fn match_token(&mut self, typ: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.peek().token_type == typ {
            self.advance();
            return true;
        }

        false
    }

    fn match_tokens(&mut self, typs: &[TokenType]) -> bool {
        for ty in typs {
            if self.match_token(*ty) {
                return true;
            }
        }

        false
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }

    fn get_id(&mut self) -> usize {
        let id: usize = self.id;
        self.id += 1;

        id
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == Semicolon {
                return;
            }

            match self.peek().token_type {
                Clazz | Fc | Let | For | If | While | Write | Return => return,
                _ => (),
            }

            self.advance();
        }
    }

    fn std_md(&self) -> HashMap<&str, Vec<&str>> {
        HashMap::from([
            ("os", vec!["exit", "current_time", "input", "name", "arch"]),
            ("math", vec!["sqrt", "E", "PI", "TAU", "pow"]),
            (
                "list",
                vec!["new", "add", "size", "reverse", "get", "pop", "remove"],
            ),
            ("utils", vec!["type", "parse"]),
            (
                "string",
                vec![
                    "length", "split", "find", "push", "replace", "trim", "trim_l", "trim_r",
                ],
            ),
        ])
    }
}
