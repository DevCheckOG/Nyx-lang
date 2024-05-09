use std::collections::HashMap;

use super::{expr::Expr, panic::PanicHandler, stmt::Stmt, tokenizer::Token, types::NyxResult};

#[derive(Copy, Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    locals: HashMap<usize, usize>,
    fc: FunctionType,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            locals: HashMap::new(),
            fc: FunctionType::None,
        }
    }

    fn resolve_internal(&mut self, stmt: &Stmt) -> NyxResult {
        match stmt {
            Stmt::Block { .. } => self.resolve_block(stmt)?,
            Stmt::Let { .. } => self.resolve_extr_var(stmt)?,
            Stmt::Const { .. } => self.resolve_extr_var(stmt)?,
            Stmt::Clazz {
                name,
                methods,
                superclass,
            } => {
                if let Some(super_expr) = superclass {
                    if let Expr::Variable {
                        id: _,
                        name: super_name,
                    } = super_expr
                    {
                        if super_name.lexeme == name.lexeme {
                            return Err(format!(
                                "Clazz cannot inherit from itself. ({}:{})",
                                name.line, name.column
                            ));
                        }
                    }

                    self.resolve_expr(super_expr)?;
                    self.begin_scope();
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(String::from("super"), true);
                }

                self.declare(name)?;
                self.define(name);

                self.begin_scope();
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert(String::from("this"), true);

                methods
                    .iter()
                    .try_for_each(|method| self.resolve_function(method, FunctionType::Method))?;

                self.end_scope();

                if superclass.is_some() {
                    self.end_scope();
                }
            }
            Stmt::Function { .. } => self.resolve_function(stmt, FunctionType::Function)?,
            Stmt::Expression { expr } => self.resolve_expr(expr)?,
            Stmt::If { .. } => self.resolve_if_stmt(stmt)?,
            Stmt::Write { exprs } => {
                exprs.iter().try_for_each(|expr| self.resolve_expr(expr))?;
            }
            Stmt::Return { keyword, value } => {
                if self.fc == FunctionType::None {
                    return Err(format!(
                        "A class cannot inherit from itself. ({}:{})",
                        keyword.line, keyword.column
                    ));
                }

                if let Some(value) = value {
                    self.resolve_expr(value)?;
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_internal(body.as_ref())?;
            }

            _ => return Ok(()),
        }
        Ok(())
    }

    fn resolve_many(&mut self, stmts: &[Stmt]) {
        stmts.iter().for_each(|stmt| {
            let _ = self.resolve_internal(stmt);
        });
    }

    pub fn resolve(mut self, stmts: &[Stmt]) -> Result<HashMap<usize, usize>, String> {
        self.resolve_many(stmts);
        Ok(self.locals)
    }

    fn resolve_block(&mut self, stmt: &Stmt) -> NyxResult {
        if let Stmt::Block { statements } = stmt {
            self.begin_scope();
            self.resolve_many(statements.as_slice());
            self.end_scope();
        } else {
            PanicHandler::new(None, None, None, "Uknown type in code block.").panic();
        }

        Ok(())
    }

    fn resolve_extr_var(&mut self, stmt: &Stmt) -> NyxResult {
        if let Stmt::Let { name, init } = stmt {
            self.declare(name)?;
            self.resolve_expr(init)?;
            self.define(name);
        } else if let Stmt::Const { name, init } = stmt {
            self.declare(name)?;
            self.resolve_expr(init)?;
            self.define(name);
        } else {
            PanicHandler::new(None, None, None, "Uknown type in variable statement.").panic();
        }

        Ok(())
    }

    fn resolve_function(&mut self, stmt: &Stmt, fn_type: FunctionType) -> NyxResult {
        if let Stmt::Function { name, params, body } = stmt {
            self.declare(name)?;
            self.define(name);
            self.resolve_function_helper(params, body.iter().as_slice(), fn_type)?;

            return Ok(());
        }

        PanicHandler::new(None, None, None, "Uknown type in function statement.").panic();

        Ok(())
    }

    fn resolve_if_stmt(&mut self, stmt: &Stmt) -> NyxResult {
        if let Stmt::If {
            predicate,
            then,
            elf,
            els,
        } = stmt
        {
            self.resolve_expr(predicate)?;
            self.resolve_internal(then)?;

            if let Some(elf) = elf {
                self.resolve_internal(elf)?;
            }

            if let Some(els) = els {
                self.resolve_internal(els)?;
            }

            return Ok(());
        }

        PanicHandler::new(None, None, None, "Uknown type in if statement.").panic();

        Ok(())
    }

    fn resolve_function_helper(
        &mut self,
        params: &[Token],
        body: &[Stmt],
        resolving_function: FunctionType,
    ) -> NyxResult {
        let enclosing_fc: FunctionType = self.fc;

        self.fc = resolving_function;

        self.begin_scope();

        params.iter().try_for_each(|param| {
            let rs: NyxResult = self.declare(param);
            self.define(param);

            rs
        })?;

        self.resolve_many(body);
        self.end_scope();
        self.fc = enclosing_fc;

        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().unwrap_or_else(|| {
            PanicHandler::new(None, None, None, "Unreachable scopes.").panic();

            HashMap::new()
        });
    }

    fn declare(&mut self, name: &Token) -> NyxResult {
        let size: usize = self.scopes.len();

        if !self.scopes.is_empty() && !self.scopes[size - 1].contains_key(&name.lexeme.to_string())
        {
            self.scopes[size - 1].insert(name.lexeme.to_string(), false);
            return Ok(());
        }

        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if !self.scopes.is_empty() {
            let size: usize = self.scopes.len();
            self.scopes[size - 1].insert(name.lexeme.to_string(), true);
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> NyxResult {
        match expr {
            Expr::Variable { id, name: _ } => self.resolve_let(expr, *id),
            Expr::Assign { id, .. } => self.resolve_assign(expr, *id),
            Expr::Binary {
                id: _,
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Call {
                id: _,
                module: _,
                call,
                paren: _,
                arguments,
            } => {
                self.resolve_expr(call.as_ref())?;

                arguments.iter().for_each(|arg| {
                    let _ = self.resolve_expr(arg);
                });

                Ok(())
            }
            Expr::Get {
                id: _,
                object,
                name: _,
            } => self.resolve_expr(object),
            Expr::Grouping { id: _, expression } => self.resolve_expr(expression),
            Expr::Literal { id: _, value: _ } => Ok(()),
            Expr::Logical {
                id: _,
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)
            }
            Expr::Set {
                id: _,
                object,
                name: _,
                value,
            } => {
                self.resolve_expr(value)?;
                self.resolve_expr(object)
            }
            Expr::This { id, keyword } => {
                if self.fc != FunctionType::Method {
                    return Err(format!(
                        "Cannot use 'this' keyword outside of a clazz. ({}:{})",
                        keyword.line, keyword.column
                    ));
                }
                self.resolve_local(keyword, *id)
            }
            Expr::Super {
                id,
                keyword,
                method: _,
            } => {
                if self.fc != FunctionType::Method {
                    return Err(format!(
                        "Cannot use 'super' keyword outside of a clazz. ({}:{})",
                        keyword.line, keyword.column
                    ));
                }
                if self.scopes.len() < 3
                    || !self.scopes[self.scopes.len() - 3].contains_key("super")
                {
                    return Err(format!(
                        "Clazz has no superclass. ({}:{})",
                        keyword.line, keyword.column
                    ));
                }
                self.resolve_local(keyword, *id)
            }
            Expr::Unary {
                id: _,
                operator: _,
                right,
            } => self.resolve_expr(right),
            Expr::AnonFunction {
                id: _,
                paren: _,
                arguments,
                body,
            } => self.resolve_function_helper(
                arguments,
                body.iter().as_slice(),
                FunctionType::Function,
            ),

            _ => Ok(()),
        }
    }

    fn resolve_let(&mut self, expr: &Expr, resolve_id: usize) -> NyxResult {
        match expr {
            Expr::Variable { id: _, name } => {
                if !self.scopes.is_empty() {
                    if let Some(false) =
                        self.scopes[self.scopes.len() - 1].get(&name.lexeme.to_string())
                    {
                        return Err(format!(
                            "Can't read a variable in its own initializer. ({}:{})",
                            name.line, name.column
                        ));
                    }
                }

                self.resolve_local(name, resolve_id)
            }
            Expr::Call {
                id: _,
                module: _,
                call,
                paren: _,
                arguments: _,
            } => match call.as_ref() {
                Expr::Variable { id: _, name } => self.resolve_local(name, resolve_id),
                _ => {
                    PanicHandler::new(
                        None,
                        None,
                        None,
                        "Unknown type in a expression of a variable.",
                    )
                    .panic();

                    Ok(())
                }
            },
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "Unknown type in a expression of a variable.",
                )
                .panic();

                Ok(())
            }
        }
    }

    fn resolve_local(&mut self, name: &Token, resolve_id: usize) -> NyxResult {
        if !self.scopes.is_empty() {
            for i in 0..=(self.scopes.len() - 1) {
                if self.scopes[i].contains_key(&name.lexeme.to_string()) {
                    self.locals.insert(resolve_id, self.scopes.len() - 1 - i);
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn resolve_assign(&mut self, expr: &Expr, rs_id: usize) -> NyxResult {
        if let Expr::Assign { id: _, name, value } = expr {
            self.resolve_expr(value)?;
            self.resolve_local(name, rs_id)?;
            return Ok(());
        }

        PanicHandler::new(None, None, None, "Unknown type in a assign.").panic();

        Ok(())
    }
}
