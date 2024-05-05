use std::rc::Rc;

use super::{expr::Expr, tokenizer::Token};

#[derive(Clone)]
pub enum Stmt {
    Expression {
        expr: Expr,
    },
    Write {
        exprs: Vec<Expr>,
    },
    Let {
        name: Token,
        init: Expr,
    },
    Const {
        name: Token,
        init: Expr,
    },
    Block {
        statements: Vec<Stmt>,
    },
    Clazz {
        name: Token,
        methods: Vec<Stmt>,
        superclass: Option<Expr>,
    },
    If {
        predicate: Expr,
        then: Rc<Stmt>,
        elf: Option<Rc<Stmt>>,
        els: Option<Rc<Stmt>>,
    },
    Elif {
        predicate: Expr,
        then: Rc<Stmt>,
    },
    While {
        condition: Expr,
        body: Rc<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },

    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Std {
        module: String,
        fc: Option<Vec<String>>,
    },

    Break {
        keyword: Token,
    },

    Continue {
        keyword: Token,
    },

    Iteration {
        var: Token,
        value: Token,
        body: Rc<Stmt>,
    },
}
