use std::{collections::HashMap, rc::Rc};

use super::super::{
    expr::{LiteralValue, NativeFunctionImpl},
    panic::PanicHandler,
};

pub struct Strings;

impl Strings {
    pub fn gen_tree_methods() -> HashMap<&'static str, NativeFunctionImpl> {
        let mut methods: HashMap<&'static str, NativeFunctionImpl> = HashMap::new();

        methods.insert(
            "length",
            NativeFunctionImpl {
                name: "length",
                fc: Rc::new(Self::length),
            },
        );

        methods.insert(
            "split",
            NativeFunctionImpl {
                name: "split",
                fc: Rc::new(Self::split),
            },
        );

        methods.insert(
            "find",
            NativeFunctionImpl {
                name: "find",
                fc: Rc::new(Self::find),
            },
        );

        methods.insert(
            "push",
            NativeFunctionImpl {
                name: "push",
                fc: Rc::new(Self::push),
            },
        );

        methods.insert(
            "replace",
            NativeFunctionImpl {
                name: "replace",
                fc: Rc::new(Self::replace),
            },
        );

        methods.insert(
            "trim",
            NativeFunctionImpl {
                name: "trim",
                fc: Rc::new(Self::trim),
            },
        );

        methods.insert(
            "trim_l",
            NativeFunctionImpl {
                name: "trim_l",
                fc: Rc::new(Self::trim_left),
            },
        );

        methods.insert(
            "trim_r",
            NativeFunctionImpl {
                name: "trim_r",
                fc: Rc::new(Self::trim_right),
            },
        );

        methods
    }

    pub fn gen_tree_constants() -> HashMap<&'static str, LiteralValue> {
        let constants: HashMap<&'static str, LiteralValue> = HashMap::new();

        constants
    }

    pub fn length(args: &[LiteralValue]) -> LiteralValue {
        if args.is_empty() {
            PanicHandler::new(
                None,
                None,
                None,
                "(string::length()) Should must have 1 arguments.",
            )
            .panic();
        }

        match &args[0] {
            LiteralValue::StringValue(s) => LiteralValue::Number(s.len() as f64),
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(string::length()) First argument must be a string.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn split(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 2 {
            PanicHandler::new(
                None,
                None,
                None,
                "(string::split()) Should must have 2 arguments.",
            )
            .panic();
        }

        match (&args[0], &args[1]) {
            (LiteralValue::StringValue(s), LiteralValue::StringValue(sp)) => {
                let mut new_list: Vec<LiteralValue> = Vec::new();

                s.split(sp).for_each(|v| {
                    new_list.push(LiteralValue::StringValue(v.to_string()));
                });

                LiteralValue::List(new_list)
            }
            (_, _) => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(string::split()) The first argument must be a string and the other second argument must also be a string.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn find(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 2 {
            PanicHandler::new(
                None,
                None,
                None,
                "(string::find()) Should must have 2 arguments.",
            )
            .panic();
        }

        match (&args[0], &args[1]) {
            (LiteralValue::StringValue(s), LiteralValue::StringValue(search)) => {
                let rs: Option<usize> = s.find(search);

                if let Some(r) = rs {
                    return LiteralValue::Number(r as f64);
                }

                LiteralValue::Null
            }

            (_, _) => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(string::find()) The first argument must be a string and the other second argument must also be a string.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn push(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 2 {
            PanicHandler::new(
                None,
                None,
                None,
                "(string::push()) Should must have 2 arguments.",
            )
            .panic();
        }

        match (args[0].clone(), args[1].clone()) {
            (LiteralValue::StringValue(mut s), LiteralValue::StringValue(v)) => {
                s.push_str(v.as_str());
                LiteralValue::StringValue(s)
            }
            (_, _) => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(string::push()) The first argument must be a string and the other second argument must also be a string.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn replace(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 3 {
            PanicHandler::new(
                None,
                None,
                None,
                "(string::replace()) Should must have 3 arguments.",
            )
            .panic();
        }

        match (&args[0], &args[1], &args[2]) {
            (
                LiteralValue::StringValue(s),
                LiteralValue::StringValue(old),
                LiteralValue::StringValue(new),
            ) => LiteralValue::StringValue(s.replace(old, new)),
            (_, _, _) => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(string::replace()) The correctly arguments are (source string, old string, new string).",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn trim(args: &[LiteralValue]) -> LiteralValue {
        if args.is_empty() {
            PanicHandler::new(
                None,
                None,
                None,
                "(string::trim()) Should must have 1 arguments.",
            )
            .panic();
        }

        match &args[0] {
            LiteralValue::StringValue(s) => LiteralValue::StringValue(s.replace(' ', "")),
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(string::trim()) The correctly arguments are (source string).",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn trim_left(args: &[LiteralValue]) -> LiteralValue {
        if args.is_empty() {
            PanicHandler::new(
                None,
                None,
                None,
                "(string::trim_l()) Should must have 1 arguments.",
            )
            .panic();
        }

        match &args[0] {
            LiteralValue::StringValue(s) => LiteralValue::StringValue(s.trim_start().to_string()),
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(string::trim_l()) The correctly arguments are (source string).",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn trim_right(args: &[LiteralValue]) -> LiteralValue {
        if args.is_empty() {
            PanicHandler::new(
                None,
                None,
                None,
                "(string::trim_r()) Should must have 1 arguments.",
            )
            .panic();
        }

        match &args[0] {
            LiteralValue::StringValue(s) => LiteralValue::StringValue(s.trim_end().to_string()),
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(string::trim_r()) The correctly arguments are (source string).",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }
}
