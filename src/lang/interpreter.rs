use std::{collections::HashMap, rc::Rc};

use super::{
    environment::Environment,
    expr::{CallableImpl, FunctionImpl, LiteralValue, NativeFunctionImpl},
    libraries::{list::List, math::Math, os::OS, strings::Strings, utils::Utils},
    panic::PanicHandler,
    stmt::Stmt,
    types::NyxResult,
};

pub struct NyxInterpreter {
    pub specials: HashMap<&'static str, LiteralValue>,
    pub environment: Environment,

    breaking: bool,
    continuing: bool,
    returning: bool,
}

impl NyxInterpreter {
    pub fn new() -> Self {
        Self {
            specials: HashMap::new(),
            environment: Environment::new(HashMap::new()),
            breaking: false,
            continuing: false,
            returning: false,
        }
    }

    pub fn resolve(&self, locals: HashMap<usize, usize>) {
        self.environment.resolve(locals);
    }

    pub fn with_env(env: Environment) -> Self {
        Self {
            specials: HashMap::new(),
            environment: env,
            breaking: false,
            continuing: false,
            returning: false,
        }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> NyxResult {
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expr } => {
                    expr.evaluate(&self.environment)?;
                }
                Stmt::Write { exprs } => {
                    for expr in exprs {
                        println!(
                            "{}",
                            expr.evaluate(&self.environment)?
                                .convert()
                                .replace("\\n", "\n")
                        );
                    }
                }
                Stmt::Let { name, init } => {
                    self.environment
                        .define(&name.lexeme, init.evaluate(&self.environment)?);
                }
                Stmt::Const { name, init } => {
                    self.environment
                        .define(&name.lexeme, init.evaluate(&self.environment)?);
                }
                Stmt::Block { statements } => {
                    let new: Environment = self.environment.enclose();
                    let old: Environment = self.environment.clone();

                    self.environment = new;
                    let block: NyxResult = self.interpret(statements.iter().collect());
                    self.environment = old;

                    block?;
                }
                Stmt::Clazz {
                    name,
                    methods,
                    superclass,
                } => {
                    let mut methods_map: HashMap<String, FunctionImpl> = HashMap::new();

                    let superclass_value: Option<Rc<LiteralValue>> = if let Some(superclass) =
                        superclass
                    {
                        let superclass: LiteralValue = superclass.evaluate(&self.environment)?;
                        if let LiteralValue::Clazz { .. } = superclass {
                            Some(Rc::new(superclass))
                        } else {
                            return Err(format!(
                                "Superclass must be a class, not ({}). ({}:{})",
                                superclass.to_type(),
                                name.line,
                                name.column
                            ));
                        }
                    } else {
                        None
                    };

                    self.environment.define(&name.lexeme, LiteralValue::Null);

                    self.environment = self.environment.enclose();

                    if let Some(sc) = superclass_value.to_owned() {
                        self.environment.define("super", (*sc).clone());
                    }

                    methods.iter().for_each(|m| {
                        if let Stmt::Function { name, .. } = m {
                            methods_map.insert(name.lexeme.clone(), self.build_fc(m));
                        } else {
                            PanicHandler::new(
                                Some(name.line),
                                Some(name.column),
                                Some(&name.lexeme),
                                "Something that was not a function was in the methods of a class.",
                            )
                            .panic();
                        }
                    });

                    if !self.environment.assign_global(
                        &name.lexeme,
                        &LiteralValue::Clazz {
                            name: name.lexeme.clone(),
                            methods: methods_map,
                            superclass: superclass_value,
                        },
                    ) {
                        return Err(format!(
                            "Class definition failed for {}. ({}:{})",
                            name.lexeme, name.line, name.column
                        ));
                    }

                    self.environment = (*self.environment.enclosing.to_owned().unwrap()).clone();
                }
                Stmt::If {
                    predicate,
                    then,
                    elf,
                    els,
                } => {
                    let truth: LiteralValue = predicate.evaluate(&self.environment)?;
                    if truth.truthy() == LiteralValue::True {
                        self.interpret(vec![then])?;
                    } else if let Some(elf_stmt) = elf {
                        self.interpret(vec![elf_stmt])?;
                    } else if let Some(els_stmt) = els {
                        self.interpret(vec![els_stmt])?;
                    }
                }
                Stmt::Elif { predicate, then } => {
                    let truth: LiteralValue = predicate.evaluate(&self.environment)?;
                    if truth.truthy() == LiteralValue::True {
                        self.interpret(vec![then])?;
                    }
                }
                Stmt::While { condition, body } => {
                    let mut flag: LiteralValue = condition.evaluate(&self.environment)?;

                    while flag.truthy() == LiteralValue::True {
                        if self.breaking {
                            break;
                        } else if self.continuing {
                            self.continuing = false;
                            continue;
                        } else if self.returning {
                            break;
                        }

                        self.interpret(vec![body])?;

                        flag = condition.evaluate(&self.environment)?;
                    }

                    self.breaking = false;
                    self.continuing = false;
                    self.returning = false;
                }

                Stmt::Iteration { var, value, body } => {
                    if let Some(v) = self.environment.get_value(value.lexeme.clone()) {
                        match v {
                            LiteralValue::List(list) => {
                                for item in list {
                                    if self.breaking {
                                        break;
                                    } else if self.continuing {
                                        self.continuing = false;
                                        continue;
                                    } else if self.returning {
                                        break;
                                    }

                                    self.environment.define(&var.lexeme, item);
                                    self.interpret(vec![body])?;
                                }

                                self.breaking = false;
                                self.continuing = false;
                                self.returning = false;
                            }

                            _ => {
                                PanicHandler::new(
                                    Some(value.line),
                                    Some(value.column),
                                    Some(&value.lexeme),
                                    "The interation value is not iterable.",
                                )
                                .panic();
                            }
                        }
                    }
                }
                Stmt::Function { name, .. } => {
                    self.environment.define(
                        &name.lexeme,
                        LiteralValue::Callable(CallableImpl::Function(self.build_fc(stmt))),
                    );
                }
                Stmt::Return { keyword: _, value } => {
                    let eval: LiteralValue = if let Some(value) = value {
                        value.evaluate(&self.environment)?
                    } else {
                        LiteralValue::Null
                    };

                    self.specials.insert("return", eval);
                    self.returning = true;
                }

                Stmt::Std { module, fc } => match &fc.is_some() {
                    true => match module.as_str() {
                        "list" => self.list(fc.clone().unwrap().as_slice()),
                        "os" => self.os(fc.clone().unwrap().as_slice()),
                        "math" => self.math(fc.clone().unwrap().as_slice()),
                        "utils" => self.utils(fc.clone().unwrap().as_slice()),
                        "string" => self.string(fc.clone().unwrap().as_slice()),

                        _ => {
                            PanicHandler::new(
                                None,
                                None,
                                None,
                                "Uknown standard module in lib declaration.",
                            )
                            .panic();
                        }
                    },

                    false => match module.as_str() {
                        "list" => self.environment.define(
                            "list",
                            LiteralValue::Module {
                                name: "list",
                                methods: List::gen_tree_methods(),
                                constants: None,
                            },
                        ),
                        "math" => self.environment.define(
                            "math",
                            LiteralValue::Module {
                                name: "math",
                                methods: Math::gen_tree_methods(),
                                constants: Some(Math::gen_tree_constants()),
                            },
                        ),
                        "os" => self.environment.define(
                            "os",
                            LiteralValue::Module {
                                name: "os",
                                methods: OS::gen_tree_methods(),
                                constants: Some(OS::gen_tree_constants()),
                            },
                        ),
                        "utils" => self.environment.define(
                            "utils",
                            LiteralValue::Module {
                                name: "utils",
                                methods: Utils::gen_tree_methods(),
                                constants: None,
                            },
                        ),

                        "string" => self.environment.define(
                            "string",
                            LiteralValue::Module {
                                name: "string",
                                methods: Strings::gen_tree_methods(),
                                constants: Some(Strings::gen_tree_constants()),
                            },
                        ),

                        _ => {
                            PanicHandler::new(
                                None,
                                None,
                                None,
                                "Uknown standard module in lib statement.",
                            )
                            .panic();
                        }
                    },
                },

                Stmt::Break { .. } => self.breaking = true,
                Stmt::Continue { .. } => self.continuing = true,
            };
        }

        Ok(())
    }

    fn string(&self, invoke: &[String]) {
        invoke.iter().for_each(|f| match f.as_str() {
            "length" => {
                self.environment
                    .define("length", self.build_native_fc("length", Strings::length));
            }

            "split" => {
                self.environment
                    .define("split", self.build_native_fc("split", Strings::split));
            }

            "find" => {
                self.environment
                    .define("find", self.build_native_fc("find", Strings::find));
            }

            "replace" => {
                self.environment
                    .define("replace", self.build_native_fc("replace", Strings::replace));
            }

            "push" => {
                self.environment
                    .define("push", self.build_native_fc("push", Strings::push));
            }

            "trim" => {
                self.environment
                    .define("trim", self.build_native_fc("trim", Strings::trim));
            }

            "trim_l" => {
                self.environment
                    .define("trim_l", self.build_native_fc("trim_l", Strings::trim_left));
            }

            "trim_r" => {
                self.environment.define(
                    "trim_r",
                    self.build_native_fc("trim_r", Strings::trim_right),
                );
            }

            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "Uknown function or constant in the importation of the module String.",
                )
                .panic();
            }
        });
    }

    fn list(&self, invoke: &[String]) {
        invoke.iter().for_each(|f| match f.as_str() {
            "new" => {
                self.environment
                    .define("new_list", self.build_native_fc("new", List::gen));
            }
            "size" => {
                self.environment
                    .define("size", self.build_native_fc("size", List::size));
            }
            "add" => {
                self.environment
                    .define("add", self.build_native_fc("add", List::add));
            }
            "reverse" => {
                self.environment
                    .define("reverse", self.build_native_fc("reverse", List::reverse));
            }
            "get" => {
                self.environment
                    .define("get", self.build_native_fc("get", List::get));
            }
            "pop" => {
                self.environment
                    .define("pop", self.build_native_fc("pop", List::pop));
            }
            "remove" => {
                self.environment
                    .define("remove", self.build_native_fc("remove", List::remove));
            }
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "Uknown function or constant in the importation of an List.",
                )
                .panic();
            }
        });
    }

    fn os(&self, invoke: &[String]) {
        invoke.iter().for_each(|f| match f.as_str() {
            "exit" => {
                self.environment
                    .define("exit", self.build_native_fc("exit", OS::exit));
            }
            "current_time" => {
                self.environment.define(
                    "current_time",
                    self.build_native_fc("current_time", OS::current_time),
                );
            }
            "input" => {
                self.environment
                    .define("input", self.build_native_fc("input", OS::input));
            }
            "name" => self.environment.define(
                "name",
                LiteralValue::StringValue(std::env::consts::OS.to_string()),
            ),
            "arch" => self.environment.define(
                "arch",
                LiteralValue::StringValue(std::env::consts::ARCH.to_string()),
            ),

            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "Uknown function or constant in the importation of an OS.",
                )
                .panic();
            }
        });
    }

    fn math(&self, invoke: &[String]) {
        invoke.iter().for_each(|f| match f.as_str() {
            "sqrt" => {
                self.environment
                    .define("sqrt", self.build_native_fc("sqrt", Math::sqrt));
            }

            "E" => self
                .environment
                .define("E", LiteralValue::Number(std::f64::consts::E)),

            "PI" => self
                .environment
                .define("PI", LiteralValue::Number(std::f64::consts::PI)),

            "TAU" => self
                .environment
                .define("TAU", LiteralValue::Number(std::f64::consts::TAU)),

            "pow" => {
                self.environment
                    .define("pow", self.build_native_fc("pow", Math::pow));
            }

            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "Uknown function or constant in the importation of an Math.",
                )
                .panic();
            }
        });
    }

    fn utils(&self, invoke: &[String]) {
        invoke.iter().for_each(|f| match f.as_str() {
            "type" => {
                self.environment
                    .define("type", self.build_native_fc("type", Utils::get_type));
            }

            "parse" => {
                self.environment
                    .define("parse", self.build_native_fc("parse", Utils::parse));
            }

            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "Uknown function or constant in the importation of the module Utils.",
                )
                .panic();
            }
        });
    }

    fn build_native_fc<F>(&self, name: &'static str, fc: F) -> LiteralValue
    where
        F: Fn(&[LiteralValue]) -> LiteralValue + 'static,
    {
        LiteralValue::Callable(CallableImpl::NativeFunction(NativeFunctionImpl {
            name,
            fc: Rc::new(fc),
        }))
    }

    fn build_fc(&self, stmt: &Stmt) -> FunctionImpl {
        if let Stmt::Function { name, params, body } = stmt {
            return FunctionImpl {
                name: name.lexeme.clone(),
                arity: params.len() as u8,
                parent_env: self.environment.clone(),
                params: params.iter().map(|t| t.to_owned()).collect::<Vec<_>>(),
                body: body.iter().map(|b| b.to_owned()).collect::<Vec<_>>(),
            };
        }

        PanicHandler::new(
            None,
            None,
            None,
            "Tried to make a function from a non-function statement.",
        )
        .panic();

        unreachable!();
    }
}
