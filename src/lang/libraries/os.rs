use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
    process::exit,
    rc::Rc,
    time::UNIX_EPOCH,
};

use super::super::{
    expr::{LiteralValue, NativeFunctionImpl},
    panic::PanicHandler,
};

pub struct OS;

impl OS {
    pub fn gen_tree_methods() -> HashMap<&'static str, NativeFunctionImpl> {
        let mut methods: HashMap<&'static str, NativeFunctionImpl> = HashMap::new();

        methods.insert(
            "exit",
            NativeFunctionImpl {
                name: "exit",
                fc: Rc::new(Self::exit),
            },
        );

        methods.insert(
            "current_time",
            NativeFunctionImpl {
                name: "current_time",
                fc: Rc::new(Self::current_time),
            },
        );

        methods.insert(
            "input",
            NativeFunctionImpl {
                name: "input",
                fc: Rc::new(Self::input),
            },
        );

        methods
    }

    pub fn gen_tree_constants() -> HashMap<&'static str, LiteralValue> {
        let mut constants: HashMap<&'static str, LiteralValue> = HashMap::new();

        constants.insert(
            "arch",
            LiteralValue::StringValue(std::env::consts::ARCH.to_string()),
        );

        constants.insert(
            "name",
            LiteralValue::StringValue(std::env::consts::OS.to_string()),
        );

        constants
    }

    pub fn exit(args: &[LiteralValue]) -> LiteralValue {
        if args.len() != 1 {
            PanicHandler::new(
                None,
                None,
                None,
                "(os::exit()) Should must have 1 argument.",
            )
            .panic();

            return LiteralValue::Null;
        }

        match args[0] {
            LiteralValue::Number(i) => {
                if (i as i32) > 0 {
                    panic!("\nNyx exit with code ({}).\n", i);
                }

                exit(i as i32);
            }
            _ => {
                PanicHandler::new(
                    None,
                    None,
                    None,
                    "(os::exit()) Should must have 1 argument of type number.",
                )
                .panic();

                LiteralValue::Null
            }
        }
    }

    pub fn current_time(_args: &[LiteralValue]) -> LiteralValue {
        let time: u128 = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("[INTERNAL ERROR] Could not get system time.")
            .as_millis();

        LiteralValue::Number(time as f64 / 1000.0)
    }

    pub fn input(args: &[LiteralValue]) -> LiteralValue {
        if args.len() >= 2 {
            PanicHandler::new(
                None,
                None,
                None,
                "(os::input()) Should must have 1 argument or less.",
            )
            .panic();

            return LiteralValue::Null;
        }

        match args.len() {
            1 => {
                let mut reader: String = String::new();

                print!("{}", args[0].convert());

                stdout().flush().ok();

                if stdin().read_line(&mut reader).is_ok() {
                    return LiteralValue::StringValue(reader.trim().to_string());
                }

                PanicHandler::new(None, None, None, "(os::input()) had an unexpected error.")
                    .panic();

                LiteralValue::Null
            }
            _ => {
                let mut reader: String = String::new();

                if stdin().read_line(&mut reader).is_ok() {
                    return LiteralValue::StringValue(reader.trim().to_string());
                }

                PanicHandler::new(None, None, None, "(os::input()) had an unexpected error.")
                    .panic();

                LiteralValue::Null
            }
        }
    }
}
