use std::{collections::HashMap, rc::Rc};

pub struct List;

use super::super::{
    expr::{LiteralValue, NativeFunctionImpl},
    panic::PanicHandler,
};

impl List {
    pub fn gen_tree_methods() -> HashMap<&'static str, NativeFunctionImpl> {
        let mut methods: HashMap<&'static str, NativeFunctionImpl> = HashMap::new();

        methods.insert(
            "add",
            NativeFunctionImpl {
                name: "add",
                fc: Rc::new(Self::add),
            },
        );

        methods.insert(
            "gen",
            NativeFunctionImpl {
                name: "gen",
                fc: Rc::new(Self::gen),
            },
        );

        methods.insert(
            "size",
            NativeFunctionImpl {
                name: "size",
                fc: Rc::new(Self::size),
            },
        );

        methods.insert(
            "reverse",
            NativeFunctionImpl {
                name: "reverse",
                fc: Rc::new(Self::reverse),
            },
        );

        methods.insert(
            "get",
            NativeFunctionImpl {
                name: "get",
                fc: Rc::new(Self::get),
            },
        );

        methods.insert(
            "pop",
            NativeFunctionImpl {
                name: "pop",
                fc: Rc::new(Self::pop),
            },
        );

        methods.insert(
            "remove",
            NativeFunctionImpl {
                name: "remove",
                fc: Rc::new(Self::remove),
            },
        );

        methods
    }

    pub fn gen(_: &[LiteralValue]) -> LiteralValue {
        LiteralValue::List(Vec::new())
    }

    pub fn add(args: &[LiteralValue]) -> LiteralValue {
        if args.len() < 2 {
            PanicHandler::new(
                None,
                None,
                None,
                "(list::add()) Should must have 2 arguments or more.",
            )
            .panic();

            return LiteralValue::Null;
        }

        match &args[0] {
            LiteralValue::List(array) => {
                let mut new: Vec<LiteralValue> = array.to_owned();
                args.iter().skip(1).for_each(|i| new.push(i.to_owned()));
                LiteralValue::List(new)
            }
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(list::add()) First argument must be an list.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn size(args: &[LiteralValue]) -> LiteralValue {
        if args.is_empty() {
            PanicHandler::new(
                None,
                None,
                None,
                "(list::size()) Should must have 1 arguments.",
            )
            .panic();

            return LiteralValue::Null;
        }

        match &args[0] {
            LiteralValue::List(list) => LiteralValue::Number(list.len() as f64),
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(list::size()) First argument must be an list.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn reverse(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 1 {
            PanicHandler::new(
                None,
                None,
                None,
                "(list::reverse()) Should must have 1 arguments.",
            )
            .panic();

            return LiteralValue::Null;
        }

        match &args[0] {
            LiteralValue::List(list) => {
                let mut new: Vec<LiteralValue> = list.clone();
                new.reverse();
                LiteralValue::List(new)
            }
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(list::reverse()) First argument must be an list.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn get(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 2 {
            PanicHandler::new(
                None,
                None,
                None,
                "(list::get()) Should must have 2 arguments.",
            )
            .panic();

            return LiteralValue::Null;
        }

        match (&args[0], &args[1]) {
            (LiteralValue::List(list), LiteralValue::Number(num)) => {
                if *num != 0.0 {
                    if let Some(i) = list.get(*num as usize - 1) {
                        return LiteralValue::List(vec![i.to_owned(), LiteralValue::Number(*num)]);
                    } else {
                        PanicHandler::new(
                            None,
                            None,
                            None,
                            "(list::get()) Index must be less than the size of the list.",
                        )
                        .panic();

                        return LiteralValue::Null;
                    }
                }

                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(list::get()) Index must be greater than 0.",
                )
                .panic();

                LiteralValue::Null
            }

            (_, _) => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(list::get()) First argument must be an list or the second argument must be a number.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn pop(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 1 {
            PanicHandler::new(
                None,
                None,
                None,
                "(list::pop()) Should must have 1 argument.",
            )
            .panic();

            return LiteralValue::Null;
        }

        match &args[0] {
            LiteralValue::List(list) => {
                let mut new: Vec<LiteralValue> = list.to_owned();
                let rs: Option<LiteralValue> = new.pop();

                if rs.is_some() {
                    return LiteralValue::List(new);
                }

                LiteralValue::Null
            }
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(list::pop()) First argument must be an list.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn remove(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 2 {
            PanicHandler::new(
                None,
                None,
                None,
                "(list::remove()) Should must have 2 arguments.",
            )
            .panic();

            return LiteralValue::Null;
        }

        match (&args[0], &args[1]) {
            (LiteralValue::List(list), LiteralValue::Number(num)) => {
                let mut new: Vec<LiteralValue> = list.to_owned();

                if new.get(*num as usize - 1).is_some() {
                    let rs: LiteralValue = new.remove(*num as usize - 1);
                    return rs;
                }

                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(list::remove()) Index must be less than the size of the list.",
                )
                .panic();

                LiteralValue::Null
            }

            (_, _) => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(list::remove()) First argument must be an list.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }
}
