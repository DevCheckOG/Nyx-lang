use std::{cell::RefCell, cmp::PartialEq, collections::HashMap, rc::Rc};

use super::{
    environment::Environment,
    interpreter::NyxInterpreter,
    panic::PanicHandler,
    stmt::Stmt,
    tokenizer,
    tokenizer::{Token, TokenType},
    types::NyxFunction,
};

#[derive(Clone)]
pub struct FunctionImpl {
    pub name: String,
    pub arity: u8,
    pub parent_env: Environment,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Clone)]
pub struct NativeFunctionImpl {
    pub name: &'static str,
    pub fc: NyxFunction,
}

#[derive(Clone)]
pub enum CallableImpl {
    Function(FunctionImpl),
    NativeFunction(NativeFunctionImpl),
}

#[derive(Clone)]
pub enum LiteralValue {
    Number(f64),
    StringValue(String),
    Callable(CallableImpl),
    True,
    False,
    Null,
    Clazz {
        name: String,
        methods: HashMap<String, FunctionImpl>,
        superclass: Option<Rc<LiteralValue>>,
    },
    ClassInstance {
        class: Rc<LiteralValue>,
        fields: Rc<RefCell<Vec<(String, LiteralValue)>>>,
    },
    Module {
        name: &'static str,
        methods: HashMap<&'static str, NativeFunctionImpl>,
        constants: Option<HashMap<&'static str, LiteralValue>>,
    },
    List(Vec<LiteralValue>),
}

pub fn run_function(
    fc: FunctionImpl,
    args: &[Expr],
    eval_env: &Environment,
) -> Result<LiteralValue, String> {
    if args.len() as u8 != fc.arity {
        PanicHandler::new(
            None,
            None,
            None,
            format!(
                "Callable ({}) expected ({}) arguments but got ({}) instead.",
                fc.name,
                fc.arity,
                args.len()
            )
            .as_str(),
        )
        .panic();

        return Ok(LiteralValue::Null);
    }

    let fc_env: Environment = fc.parent_env.enclose();

    let mut parsed_args: Vec<LiteralValue> = Vec::with_capacity(args.len());

    for arg in args {
        if let Ok(literal) = arg.evaluate(eval_env) {
            parsed_args.push(literal);
        }
    }

    parsed_args.iter().enumerate().for_each(|(i, val)| {
        fc_env.define(&fc.params[i].lexeme, val.clone());
    });

    let mut inter: NyxInterpreter = NyxInterpreter::with_env(fc_env);

    for i in 0..(fc.body.len()) {
        inter.interpret(vec![&fc.body[i]])?;

        if let Some(value) = inter.specials.get("return") {
            return Ok(value.to_owned());
        }
    }

    Ok(LiteralValue::Null)
}

pub fn find_method(name: &str, class: LiteralValue) -> Option<FunctionImpl> {
    if let LiteralValue::Clazz {
        name: _,
        methods,
        superclass,
    } = class
    {
        if let Some(fun) = methods.get(name) {
            return Some(fun.to_owned());
        } else if let Some(superclass) = superclass {
            return find_method(name, (*superclass).clone());
        }

        return None;
    }

    PanicHandler::new(None, None, None, "Cannot find method on non-class.").panic();

    None
}

impl LiteralValue {
    pub fn convert(&self) -> String {
        match self {
            LiteralValue::Number(x) => x.to_string(),
            LiteralValue::StringValue(x) => x.to_string(),
            LiteralValue::True => "true".to_string(),
            LiteralValue::False => "false".to_string(),
            LiteralValue::Null => "null".to_string(),
            LiteralValue::Callable(CallableImpl::Function(FunctionImpl {
                name, arity, ..
            })) => format!("{name}/{arity}"),
            LiteralValue::Callable(CallableImpl::NativeFunction(NativeFunctionImpl {
                name,
                ..
            })) => name.to_string(),
            LiteralValue::Clazz {
                name,
                methods: _,
                superclass: _,
            } => format!("Clazz '{name}'"),
            LiteralValue::ClassInstance { class, fields: _ } => {
                if let LiteralValue::Clazz {
                    name,
                    methods: _,
                    superclass: _,
                } = &**class
                {
                    format!("Clazz instance '{name}'")
                } else {
                    PanicHandler::new(None, None, None, "Unreachable clazz name.").panic();

                    String::new()
                }
            }

            LiteralValue::List(v) => {
                if !v.is_empty() {
                    return format!(
                        "[{}]",
                        v.iter().map(|x| x.convert()).collect::<Vec<_>>().join(", ")
                    );
                }

                "[]".to_string()
            }

            LiteralValue::Module {
                name,
                methods: _,
                constants: _,
            } => format!("Module '{name}'"),
        }
    }

    pub fn to_type(&self) -> &str {
        match self {
            LiteralValue::Number(_) => "number",
            LiteralValue::Callable(_) => "callable",
            LiteralValue::StringValue(_) => "string",
            LiteralValue::True => "boolean",
            LiteralValue::False => "boolean",
            LiteralValue::Null => "null",
            LiteralValue::Clazz {
                name: _,
                methods: _,
                superclass: _,
            } => "Clazz",
            LiteralValue::ClassInstance { class, .. } => {
                if let LiteralValue::Clazz {
                    name,
                    methods: _,
                    superclass: _,
                } = &**class
                {
                    name.as_str()
                } else {
                    PanicHandler::new(None, None, None, "Unreachable clazz name.").panic();

                    ""
                }
            }

            LiteralValue::List(_) => "list",
            LiteralValue::Module { .. } => "module",
        }
    }

    pub fn from_token(tk: Token) -> Self {
        match tk.token_type {
            TokenType::Number => {
                if let Some(tokenizer::LiteralValue::FValue(x)) = tk.literal {
                    return Self::Number(x);
                }

                PanicHandler::new(None, None, None, "Could not parse number.").panic();

                Self::Number(0.0_f64)
            }

            TokenType::StringLit => {
                if let Some(tokenizer::LiteralValue::SValue(x)) = tk.literal {
                    return Self::StringValue(x);
                }

                PanicHandler::new(None, None, None, "Could not parse number.").panic();

                Self::Number(0.0_f64)
            }
            TokenType::False => Self::False,
            TokenType::True => Self::True,
            TokenType::Null => Self::Null,
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    format!(
                        "Could not convert native type to literal. ({}:{})",
                        tk.line, tk.column
                    )
                    .as_str(),
                )
                .panic();

                Self::Null
            }
        }
    }

    #[inline(always)]
    fn bool(b: bool) -> Self {
        if b {
            return LiteralValue::True;
        }

        LiteralValue::False
    }

    fn is_false(&self) -> LiteralValue {
        match self {
            LiteralValue::Number(x) => {
                if *x == 0.0_f64 {
                    return LiteralValue::True;
                }

                LiteralValue::False
            }
            LiteralValue::StringValue(s) => {
                if s.is_empty() {
                    return LiteralValue::True;
                }

                LiteralValue::False
            }
            LiteralValue::True => LiteralValue::False,
            LiteralValue::False => LiteralValue::True,
            LiteralValue::Null => LiteralValue::True,
            LiteralValue::Callable(_) => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "A Callable should not be used as a boolean value.",
                )
                .panic();

                LiteralValue::Null
            }
            LiteralValue::Clazz { .. } => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "A Clazz should not be used as a boolean value.",
                )
                .panic();

                LiteralValue::Null
            }
            _ => {
                PanicHandler::new(None, None, None, "Object is not valid as a boolean value.")
                    .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn truthy(&self) -> LiteralValue {
        match self {
            LiteralValue::Number(x) => {
                if *x == 0.0_f64 {
                    return LiteralValue::False;
                }

                LiteralValue::True
            }
            LiteralValue::StringValue(s) => {
                if s.is_empty() {
                    return LiteralValue::False;
                }

                LiteralValue::True
            }
            LiteralValue::True => LiteralValue::True,
            LiteralValue::False => LiteralValue::False,
            LiteralValue::Null => LiteralValue::False,
            LiteralValue::Callable(_) => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "A Callable should not be used as a boolean value.",
                )
                .panic();

                LiteralValue::Null
            }
            LiteralValue::Clazz { .. } => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "A Clazz should not be used as a boolean value.",
                )
                .panic();

                LiteralValue::Null
            }
            _ => {
                PanicHandler::new(None, None, None, "Object is not valid as a boolean value.")
                    .panic();

                LiteralValue::Null
            }
        }
    }
}

#[derive(Clone)]
pub enum Expr {
    AnonFunction {
        id: usize,
        paren: Token,
        arguments: Vec<Token>,
        body: Vec<Stmt>,
    },
    Assign {
        id: usize,
        name: Token,
        value: Rc<Expr>,
    },
    Binary {
        id: usize,
        left: Rc<Expr>,
        operator: Token,
        right: Rc<Expr>,
    },
    Call {
        id: usize,
        module: Option<String>,
        call: Rc<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        id: usize,
        object: Rc<Expr>,
        name: Token,
    },
    Grouping {
        id: usize,
        expression: Rc<Expr>,
    },
    Literal {
        id: usize,
        value: LiteralValue,
    },
    Logical {
        id: usize,
        left: Rc<Expr>,
        operator: Token,
        right: Rc<Expr>,
    },
    Set {
        id: usize,
        object: Rc<Expr>,
        name: Token,
        value: Rc<Expr>,
    },
    This {
        id: usize,
        keyword: Token,
    },
    Super {
        id: usize,
        keyword: Token,
        method: Token,
    },
    Unary {
        id: usize,
        operator: Token,
        right: Rc<Expr>,
    },
    Variable {
        id: usize,
        name: Token,
    },

    ModuleProperty {
        id: usize,
        module: String,
        name: Token,
    },
}

impl Expr {
    pub fn get_id(&self) -> usize {
        match self {
            Expr::AnonFunction { id, .. } => *id,
            Expr::Assign { id, .. } => *id,
            Expr::Binary { id, .. } => *id,
            Expr::Call { id, .. } => *id,
            Expr::Get { id, .. } => *id,
            Expr::Grouping { id, .. } => *id,
            Expr::Literal { id, .. } => *id,
            Expr::Logical { id, .. } => *id,
            Expr::Set { id, .. } => *id,
            Expr::This { id, keyword: _ } => *id,
            Expr::Super { id, .. } => *id,
            Expr::Unary { id, .. } => *id,
            Expr::Variable { id, name: _ } => *id,
            Expr::ModuleProperty { id, .. } => *id,
        }
    }

    #[allow(dead_code)]
    pub fn convert(&self) -> String {
        match self {
            Expr::AnonFunction {
                id: _,
                paren: _,
                arguments,
                body: _,
            } => format!("anon/{}", arguments.len()),
            Expr::Assign { id: _, name, value } => format!("({name:?} = {}", value.convert()),
            Expr::Binary {
                id: _,
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                operator.lexeme,
                left.convert(),
                right.convert()
            ),
            Expr::Call {
                id: _,
                call,
                module: _,
                paren: _,
                arguments: _,
            } => format!("({})", call.convert()),
            Expr::Get {
                id: _,
                object,
                name,
            } => format!("(get {} {})", object.convert(), name.lexeme),
            Expr::Grouping { id: _, expression } => {
                format!("(group {})", expression.convert())
            }
            Expr::Literal { id: _, value } => value.convert(),
            Expr::Logical {
                id: _,
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                format_args!(
                    "{:?} {} {:?}",
                    operator.token_type, operator.lexeme, operator.literal
                ),
                left.convert(),
                right.convert()
            ),
            Expr::Set {
                id: _,
                object,
                name,
                value,
            } => format!(
                "(set {} {} {})",
                object.convert(),
                format_args!("{:?} {} {:?}", name.token_type, name.lexeme, name.literal),
                value.convert()
            ),
            Expr::This { .. } => "(this)".to_string(),
            Expr::Super {
                id: _,
                keyword: _,
                method,
            } => format!("(super {})", method.lexeme),
            Expr::Unary {
                id: _,
                operator,
                right,
            } => {
                let op: String = operator.lexeme.to_owned();
                let rhs: String = right.convert();
                format!("({} {})", op, rhs)
            }
            Expr::Variable { id: _, name } => format!("(let {})", name.lexeme),

            Expr::ModuleProperty {
                id: _,
                module: _,
                name,
            } => format!("(Module property {})", name.lexeme),
        }
    }

    pub fn evaluate(&self, environment: &Environment) -> Result<LiteralValue, String> {
        match self {
            Expr::AnonFunction {
                id: _,
                paren: _,
                arguments,
                body,
            } => Ok(LiteralValue::Callable(CallableImpl::Function(
                FunctionImpl {
                    name: String::from("anon_fc"),
                    arity: arguments.len() as u8,
                    parent_env: environment.clone(),
                    params: arguments.to_vec(),
                    body: body.to_vec(),
                },
            ))),
            Expr::Assign { id: _, name, value } => {
                let new: LiteralValue = value.evaluate(environment)?;

                if environment.constant(&name.lexeme) {
                    PanicHandler::new(
                        Some(name.line),
                        Some(name.column),
                        Some(&name.lexeme),
                        "A constant is not allowed to be reassigned.",
                    )
                    .panic();
                } else if environment.assign(&name.lexeme, &new, self.get_id()) {
                    return Ok(new);
                }

                PanicHandler::new(
                    Some(name.line),
                    Some(name.column),
                    Some(&name.lexeme),
                    "The variable has not been declared.",
                )
                .panic();

                Ok(LiteralValue::Null)
            }

            Expr::Variable { id: _, name } => match environment.get(&name.lexeme, self.get_id()) {
                Some(value) => Ok(value),
                None => {
                    PanicHandler::new(
                        Some(name.line),
                        Some(name.column),
                        Some(&name.lexeme),
                        "A Variable || Callable || Clazz || Module has not been declared.",
                    )
                    .panic();

                    Ok(LiteralValue::Null)
                }
            },

            Expr::ModuleProperty { id, module, name } => {
                if let Some(md) = environment.get(module, *id) {
                    match md {
                        LiteralValue::Module {
                            name: _,
                            methods: _,
                            constants,
                        } => {
                            if let Some(module_constants) = constants {
                                if let Some(value) = module_constants.get(&name.lexeme.as_str()) {
                                    return Ok(value.to_owned());
                                }
                            }

                            PanicHandler::new(
                                Some(name.line),
                                Some(name.column),
                                Some(module),
                                "Unknown constant in standard library module.",
                            )
                            .panic();
                        }
                        _ => {
                            PanicHandler::new(
                                Some(name.line),
                                Some(name.column),
                                Some(module),
                                "Unknown module in standard library.",
                            )
                            .panic();
                        }
                    }
                }

                PanicHandler::new(
                    Some(name.line),
                    Some(name.column),
                    Some(module),
                    "Unknown module in standard library.",
                )
                .panic();

                Ok(LiteralValue::Null)
            }

            Expr::Call {
                id,
                call,
                module,
                paren,
                arguments,
            } => {
                let callable: LiteralValue = call.evaluate(environment)?;

                match module {
                    Some(module) => match callable {
                        LiteralValue::StringValue(s) => {
                            if let Some(md) = environment.get(module, *id) {
                                match md {
                                    LiteralValue::Module {
                                        name: _, methods, ..
                                    } => {
                                        if let Some(nativefc) = methods.get(s.as_str()) {
                                            let mut eval_args: Vec<LiteralValue> = Vec::new();

                                            arguments.iter().try_for_each(|arg| {
                                                match arg.evaluate(environment) {
                                                    Ok(v) => {
                                                        eval_args.push(v);
                                                        Ok(())
                                                    }
                                                    Err(any) => Err(any),
                                                }
                                            })?;

                                            return Ok((nativefc.fc)(&eval_args));
                                        }

                                        PanicHandler::new(
                                            Some(paren.line),
                                            Some(paren.column),
                                            Some(&s),
                                            "Unknown method of a module of the standard library.",
                                        )
                                        .panic();
                                    }

                                    _ => {
                                        PanicHandler::new(
                                            Some(paren.line),
                                            Some(paren.column),
                                            Some(&s),
                                            "Unknown module in standard library.",
                                        )
                                        .panic();
                                    }
                                }
                            }

                            PanicHandler::new(
                                Some(paren.line),
                                Some(paren.column),
                                Some(&s),
                                "Unknown module in standard library.",
                            )
                            .panic();

                            Ok(LiteralValue::Null)
                        }

                        _ => {
                            PanicHandler::new(
                                Some(paren.line),
                                Some(paren.column),
                                Some(&callable.convert()),
                                "Any Object is not callable.",
                            )
                            .panic();

                            Ok(LiteralValue::Null)
                        }
                    },

                    None => match callable.clone() {
                        LiteralValue::Callable(CallableImpl::Function(fc)) => {
                            run_function(fc, arguments, environment)
                        }
                        LiteralValue::Callable(CallableImpl::NativeFunction(nativefc)) => {
                            let mut eval_args: Vec<LiteralValue> = Vec::new();

                            arguments.iter().try_for_each(|arg| {
                                match arg.evaluate(environment) {
                                    Ok(v) => {
                                        eval_args.push(v);
                                        Ok(())
                                    }
                                    Err(any) => Err(any),
                                }
                            })?;

                            Ok((nativefc.fc)(&eval_args))
                        }
                        LiteralValue::Clazz {
                            name,
                            methods,
                            superclass: _,
                        } => {
                            let instance: LiteralValue = LiteralValue::ClassInstance {
                                class: Rc::new(callable),
                                fields: Rc::new(RefCell::new(vec![])),
                            };

                            if let Some(init_method) = methods.get("init") {
                                if init_method.arity != arguments.len() as u8 {
                                    PanicHandler::new(
                                        Some(paren.line),
                                        Some(paren.column),
                                        Some(&name),
                                        "The clazz expected more arguments.",
                                    )
                                    .panic();
                                }

                                let mut init: FunctionImpl = init_method.to_owned();

                                init.parent_env = init_method.parent_env.enclose();
                                init.parent_env.define("this", instance.clone());

                                run_function(init, arguments, environment)?;
                            }

                            Ok(instance)
                        }
                        _ => {
                            PanicHandler::new(
                                Some(paren.line),
                                Some(paren.column),
                                Some(&callable.convert()),
                                "Any Object is not callable.",
                            )
                            .panic();

                            Ok(LiteralValue::Null)
                        }
                    },
                }
            }
            Expr::Literal { id: _, value } => Ok(value.to_owned()),

            Expr::Logical {
                id: _,
                left,
                operator,
                right,
            } => match operator.token_type {
                TokenType::Or => {
                    let lhs: LiteralValue = left.evaluate(environment)?;
                    if lhs.truthy() == LiteralValue::True {
                        return Ok(lhs);
                    }

                    right.evaluate(environment)
                }
                TokenType::And => {
                    let lhs: LiteralValue = left.evaluate(environment)?;
                    if lhs.truthy() == LiteralValue::False {
                        return Ok(lhs.truthy());
                    }

                    right.evaluate(environment)
                }
                _ => {
                    PanicHandler::new(
                        Some(operator.line),
                        Some(operator.column),
                        Some(&operator.lexeme),
                        "Uknown logical operator.",
                    )
                    .panic();

                    Ok(LiteralValue::Null)
                }
            },
            Expr::Get {
                id: _,
                object,
                name,
            } => {
                let obj_value: LiteralValue = object.evaluate(environment)?;

                if let LiteralValue::ClassInstance { class, fields } = obj_value.clone() {
                    for (field_name, value) in (*fields.borrow()).iter() {
                        if *field_name == name.lexeme {
                            return Ok(value.to_owned());
                        }
                    }

                    if let LiteralValue::Clazz {
                        name: _,
                        methods: _,
                        superclass: _,
                    } = *class
                    {
                        if let Some(method) = find_method(&name.lexeme, (*class).clone()) {
                            let mut callable_impl: FunctionImpl = method;

                            let new_env = callable_impl.parent_env.enclose();

                            new_env.define("this", obj_value);

                            callable_impl.parent_env = new_env;

                            return Ok(LiteralValue::Callable(CallableImpl::Function(
                                callable_impl,
                            )));
                        }
                    }

                    PanicHandler::new(
                        Some(name.line),
                        Some(name.column),
                        Some(&name.lexeme),
                        "The clazz field on an instance was not a clazz.",
                    )
                    .panic();
                }
                PanicHandler::new(
                    Some(name.line),
                    Some(name.column),
                    Some(&name.lexeme),
                    "The object does not contain this property.",
                )
                .panic();

                Ok(LiteralValue::Null)
            }
            Expr::Set {
                id: _,
                object,
                name,
                value,
            } => {
                let obj_v: LiteralValue = object.evaluate(environment)?;
                if let LiteralValue::ClassInstance { class: _, fields } = obj_v {
                    let value: LiteralValue = value.evaluate(environment)?;

                    let mut idx: usize = 0;
                    let mut found: bool = false;

                    for i in 0..(*fields.borrow()).len() {
                        let field_name: &str = &(*fields.borrow())[i].0;
                        if field_name == name.lexeme {
                            idx = i;
                            found = true;
                            break;
                        }
                    }

                    if found {
                        (*fields.borrow_mut())[idx].1 = value.to_owned();
                    } else {
                        (*fields.borrow_mut()).push((name.lexeme.to_owned(), value));
                    }

                    return Ok(LiteralValue::Null);
                }

                PanicHandler::new(
                    Some(name.line),
                    Some(name.column),
                    Some(&name.lexeme),
                    "The object does not contain this property.",
                )
                .panic();

                Ok(LiteralValue::Null)
            }
            Expr::This { id: _, keyword } => {
                let this: LiteralValue =
                    environment.get("this", self.get_id()).unwrap_or_else(|| {
                        PanicHandler::new(
                            Some(keyword.line),
                            Some(keyword.column),
                            Some(&keyword.lexeme),
                            "Couldn't lookup 'super'.",
                        )
                        .panic();

                        LiteralValue::Null
                    });
                Ok(this)
            }
            Expr::Super {
                id: _,
                keyword: _,
                method,
            } => {
                let superclass: LiteralValue =
                    environment.get("super", self.get_id()).unwrap_or_else(|| {
                        PanicHandler::new(
                            Some(method.line),
                            Some(method.column),
                            Some(&method.lexeme),
                            "Couldn't lookup 'super'.",
                        )
                        .panic();

                        LiteralValue::Null
                    });

                let instance: LiteralValue = environment.get_this_instance(self.get_id()).unwrap();

                if let LiteralValue::Clazz {
                    name,
                    methods,
                    superclass: _,
                } = superclass
                {
                    if let Some(method_value) = methods.get(&method.lexeme) {
                        method_value.clone().parent_env = method_value.parent_env.enclose();
                        method_value.parent_env.define("this", instance);
                        return Ok(LiteralValue::Callable(CallableImpl::Function(
                            method_value.to_owned(),
                        )));
                    }
                    PanicHandler::new(
                        Some(method.line),
                        Some(method.column),
                        Some(&name),
                        "No method named on the superclass.",
                    )
                    .panic();
                }

                PanicHandler::new(
                    None,
                    None,
                    None,
                    "The superclass field on an instance was not a clazz.",
                )
                .panic();

                Ok(LiteralValue::Null)
            }
            Expr::Grouping { id: _, expression } => expression.evaluate(environment),
            Expr::Unary {
                id: _,
                operator,
                right,
            } => match (&right.evaluate(environment)?, operator.token_type) {
                (LiteralValue::Number(x), TokenType::Minus) => Ok(LiteralValue::Number(-x)),
                (_, TokenType::Minus) => {
                    PanicHandler::new(
                        None,
                        None,
                        None,
                        format!(
                            "Minus not implemented. ({}:{})",
                            operator.line, operator.column
                        )
                        .as_str(),
                    )
                    .panic();

                    Ok(LiteralValue::Null)
                }
                (any, TokenType::Bang) => Ok(any.is_false()),
                (_, type_) => Err(format!(
                    "({:?}) is not a valid operator. ({}:{})",
                    type_, operator.line, operator.column
                )),
            },

            Expr::Binary {
                id: _,
                left,
                operator,
                right,
            } => {
                match (
                    &left.evaluate(environment)?,
                    operator.token_type,
                    &right.evaluate(environment)?,
                ) {
                    (LiteralValue::Number(x), TokenType::Plus, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x + y))
                    }
                    (LiteralValue::Number(x), TokenType::Minus, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x - y))
                    }
                    (LiteralValue::Number(x), TokenType::Star, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x * y))
                    }
                    (LiteralValue::Number(x), TokenType::Slash, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x / y))
                    }
                    (LiteralValue::Number(x), TokenType::Greater, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::bool(x > y))
                    }
                    (LiteralValue::Number(x), TokenType::GreaterEqual, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::bool(x >= y))
                    }
                    (LiteralValue::Number(x), TokenType::Less, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::bool(x < y))
                    }
                    (LiteralValue::Number(x), TokenType::LessEqual, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::bool(x <= y))
                    }

                    (LiteralValue::StringValue(_), op, LiteralValue::Number(_))
                    | (LiteralValue::Number(_), op, LiteralValue::StringValue(_)) => {
                        PanicHandler::new(
                            None,
                            None,
                            None,
                            format!("({:?}) is not defined for string and number.", op).as_str(),
                        )
                        .panic();

                        Ok(LiteralValue::Null)
                    }

                    (
                        LiteralValue::StringValue(s1),
                        TokenType::Plus,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::StringValue(format!("{}{}", s1, s2))),

                    (x, TokenType::BangEqual, y) => Ok(LiteralValue::bool(x != y)),
                    (x, TokenType::EqualEqual, y) => Ok(LiteralValue::bool(x == y)),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::Greater,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::bool(s1 > s2)),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::GreaterEqual,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::bool(s1 >= s2)),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::Less,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::bool(s1 < s2)),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::LessEqual,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::bool(s1 <= s2)),
                    (x, _type_, y) => {
                        PanicHandler::new(
                            None,
                            None,
                            None,
                            format!(
                                "({}) is not implemented for operands ({}) and ({}).",
                                operator.lexeme,
                                x.convert(),
                                y.convert()
                            )
                            .as_str(),
                        )
                        .panic();

                        Ok(LiteralValue::Null)
                    }
                }
            }
        }
    }
}

impl PartialEq for LiteralValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LiteralValue::Number(x), LiteralValue::Number(y)) => x == y,
            (
                LiteralValue::Callable(CallableImpl::Function(FunctionImpl {
                    name, arity, ..
                })),
                LiteralValue::Callable(CallableImpl::Function(FunctionImpl {
                    name: name2,
                    arity: arity2,
                    ..
                })),
            ) => name == name2 && arity == arity2,
            (
                LiteralValue::Callable(CallableImpl::NativeFunction(NativeFunctionImpl {
                    name,
                    ..
                })),
                LiteralValue::Callable(CallableImpl::NativeFunction(NativeFunctionImpl {
                    name: name2,
                    ..
                })),
            ) => name == name2,
            (LiteralValue::StringValue(x), LiteralValue::StringValue(y)) => x == y,
            (LiteralValue::True, LiteralValue::True) => true,
            (LiteralValue::False, LiteralValue::False) => true,
            (LiteralValue::Null, LiteralValue::Null) => true,
            _ => false,
        }
    }
}
