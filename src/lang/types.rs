use std::rc::Rc;

use super::{expr::LiteralValue, stmt::Stmt, tokenizer::Token};

pub type NyxResult<'a> = Result<(), String>;
pub type NyxAnalyzeResult<'a> = Result<&'a Vec<Token>, String>;
pub type NyxParserResult<'a> = Result<&'a Vec<Stmt>, String>;
pub type NyxInternalParserResult = Result<Stmt, String>;

pub type NyxFunction = Rc<dyn Fn(&[LiteralValue]) -> LiteralValue>;
