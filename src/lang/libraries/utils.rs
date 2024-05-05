use std::{collections::HashMap, rc::Rc};

use super::super::{
    expr::{LiteralValue, NativeFunctionImpl},
    panic::PanicHandler,
};

pub struct Utils;

impl Utils {
    pub fn gen_tree_methods() -> HashMap<&'static str, NativeFunctionImpl> {
        let mut methods: HashMap<&'static str, NativeFunctionImpl> = HashMap::new();

        methods.insert(
            "type",
            NativeFunctionImpl {
                name: "type",
                fc: Rc::new(Self::get_type),
            },
        );

        methods.insert(
            "parse",
            NativeFunctionImpl {
                name: "parse",
                fc: Rc::new(Self::parse),
            },
        );

        methods
    }

    pub fn get_type(args: &[LiteralValue]) -> LiteralValue {
        if args.is_empty() {
            PanicHandler::new(
                None,
                None,
                None,
                "(utils::type()) Should must have 1 argument.",
            )
            .panic();

            return LiteralValue::Null;
        }

        LiteralValue::StringValue(args[0].to_type().to_string())
    }

    pub fn parse(args: &[LiteralValue]) -> LiteralValue {
        if args.is_empty() {
            PanicHandler::new(
                None,
                None,
                None,
                "(utils::parse()) Should must have 1 argument.",
            )
            .panic();
        }

        match &args[0] {
            LiteralValue::StringValue(s) => {
                if let Ok(n) = s.parse::<f64>() {
                    return LiteralValue::Number(n);
                }

                LiteralValue::Null
            }
            LiteralValue::Number(n) => LiteralValue::StringValue(n.to_string()),
            _ => LiteralValue::Null,
        }
    }
}
