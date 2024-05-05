use std::{collections::HashMap, rc::Rc};

use super::super::{
    expr::{LiteralValue, NativeFunctionImpl},
    panic::PanicHandler,
};

pub struct Math;

impl Math {
    pub fn gen_tree_methods() -> HashMap<&'static str, NativeFunctionImpl> {
        let mut methods: HashMap<&'static str, NativeFunctionImpl> = HashMap::new();

        methods.insert(
            "sqrt",
            NativeFunctionImpl {
                name: "sqrt",
                fc: Rc::new(Self::sqrt),
            },
        );

        methods.insert(
            "pow",
            NativeFunctionImpl {
                name: "pow",
                fc: Rc::new(Self::pow),
            },
        );

        methods
    }

    pub fn gen_tree_constants() -> HashMap<&'static str, LiteralValue> {
        let mut constants: HashMap<&'static str, LiteralValue> = HashMap::new();

        constants.insert("PI", LiteralValue::Number(std::f64::consts::PI));
        constants.insert("E", LiteralValue::Number(std::f64::consts::E));
        constants.insert("TAU", LiteralValue::Number(std::f64::consts::TAU));

        constants
    }

    pub fn sqrt(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 1 {
            PanicHandler::new(
                None,
                None,
                None,
                "(math::sqrt()) Should must have 1 arguments.",
            )
            .panic();

            return LiteralValue::Null;
        }

        match args[0] {
            LiteralValue::Number(i) => {
                if i < 0.0 {
                    PanicHandler::new(
                        None,
                        None,
                        None,
                        "(math::sqrt()) Should must have 1 argument of type number greater than 0.",
                    )
                    .panic();

                    return LiteralValue::Null;
                }

                LiteralValue::Number(i.sqrt())
            }
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(math::sqrt()) Should must have 1 argument of type number.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn pow(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 2 {
            PanicHandler::new(
                None,
                None,
                None,
                "(math::pow()) Should must have 2 arguments.",
            )
            .panic();

            return LiteralValue::Null;
        }

        match (&args[0], &args[1]) {
            (LiteralValue::Number(x), LiteralValue::Number(y)) => {
                if *y < 0.0 {
                    PanicHandler::new(
                        None,
                        None,
                        None,
                        "(math::pow()) Should must have 2 arguments of type number greater than 0.",
                    )
                    .panic();

                    return LiteralValue::Null;
                }

                let rs: f64 = x.powf(*y);

                if rs.is_infinite() {
                    return LiteralValue::StringValue("infinite".to_string());
                }

                LiteralValue::Number(rs)
            }
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(math::pow()) Should must have 2 arguments of type number.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }
}
