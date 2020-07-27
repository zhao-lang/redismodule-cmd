extern crate redis_module;

use std::any::Any;
use std::collections::HashMap;
use redis_module::{RedisError};

#[derive(Debug, PartialEq)]
pub struct Command {
    pub name: String,
    args: Vec<Arg>,
}

impl Command {
    pub fn new(name: String) -> Self {
        Command {name, args: Vec::new()}
    }

    pub fn add_arg(&mut self, arg: Arg) {
        self.args.push(arg);
    }

    pub fn parse_args(&mut self, raw_args: Vec<String>) -> Result<HashMap<String, Box<dyn Any>>, RedisError> {
        let mut raw_args = raw_args.into_iter();
        match raw_args.next() {
           Some(cmd_name) => {
               if cmd_name != self.name {
                   return Err(RedisError::String(format!("Expected {}, got {}", self.name, cmd_name)))
               }
           },
           None => return Err(RedisError::WrongArity)
        }
        
        let res = HashMap::new();

        // // parse args
        // let next_arg = raw_args.next();
        // for arg in self.args.iter() {
        //     if arg.optional {
        //         if next_arg.is_none() {
        //             continue
        //         }
        //         if *next_arg.as_ref().unwrap() != arg.arg {
        //             continue
        //         }
        //     } else {
        //         if next_arg.is_none() {
        //             return Err(RedisError::WrongArity)
        //         }
        //     }
        //     let arg_name = next_arg.as_ref().unwrap().to_owned();

        //     let next_val = raw_args.next();
        //     if next_val.is_none() {
        //         return Err(RedisError::WrongArity)
        //     }
        //     let next_val = next_val.unwrap();
        //     if next_val.type_id() != arg.argtype {
        //         return Err(RedisError::String(format!("Expected {:?}, got {:?}", arg.argtype, next_val.type_id())))
        //     }

        // }

        Ok(res)
    }
}

#[derive(Debug, PartialEq)]
pub enum Arg {
    StringArg(StringArg),
    UintArg(UintArg),
    IntArg(IntArg),
    FloatArg(FloatArg),
}


#[derive(Debug, PartialEq)]
pub struct StringArg {
    pub arg: String,
    pub optional: bool,
    pub default: Option<String>,
}

impl StringArg {
    pub fn new(arg: String, optional: bool, default: Option<String>) -> Self {
        StringArg {arg, optional, default}
    }
}

#[derive(Debug, PartialEq)]
pub struct UintArg {
    pub arg: String,
    pub optional: bool,
    pub default: Option<u64>,
}

impl UintArg {
    pub fn new(arg: String, optional: bool, default: Option<u64>) -> Self {
        UintArg {arg, optional, default}
    }
}

#[derive(Debug, PartialEq)]
pub struct IntArg {
    pub arg: String,
    pub optional: bool,
    pub default: Option<i64>,
}

impl IntArg {
    pub fn new(arg: String, optional: bool, default: Option<i64>) -> Self {
        IntArg {arg, optional, default}
    }
}

#[derive(Debug, PartialEq)]
pub struct FloatArg {
    pub arg: String,
    pub optional: bool,
    pub default: Option<f64>,
}

impl FloatArg {
    pub fn new(arg: String, optional: bool, default: Option<f64>) -> Self {
        FloatArg {arg, optional, default}
    }
}

#[macro_export]
macro_rules! argument {
    ([
        $arg:expr,
        $type:ident,
        $optional:expr,
        $default:expr
    ]) => {
        $crate::Arg::$type($crate::$type::new($arg.to_owned(), $optional, $default))
    };
}

#[macro_export]
macro_rules! command {
    (
        name: $name:expr,
        args: [
            $($arg:tt),* $(,)*
        ] $(,)*
    ) => {{
        let mut cmd = $crate::Command::new($name);
        $(
            let arg = argument!($arg);
            cmd.add_arg(arg);
        )*
        cmd
    }};
}

#[cfg(test)]
mod tests {
    use super::{Arg, StringArg, UintArg, IntArg, FloatArg, Command};

    #[test]
    fn macro_test() {
        let cmd = command!{
            name: "test".to_owned(),
            args: [
                ["stringarg", StringArg, true, None],
                ["uintarg", UintArg, false, Some(1_u64)],
                ["intarg", IntArg, false, Some(1_i64)],
                ["floatarg", FloatArg, false, Some(1_f64)],
            ],
        };

        let mut exp = Command::new("test".to_owned());
        let arg1 = StringArg::new("stringarg".to_owned(), true, None);
        let arg2 = UintArg::new("uintarg".to_owned(), false, Some(1_u64));
        let arg3 = IntArg::new("intarg".to_owned(), false, Some(1_i64));
        let arg4 = FloatArg::new("floatarg".to_owned(), false, Some(1_f64));
        exp.add_arg(Arg::StringArg(arg1));
        exp.add_arg(Arg::UintArg(arg2));
        exp.add_arg(Arg::IntArg(arg3));
        exp.add_arg(Arg::FloatArg(arg4));

        assert_eq!(cmd, exp);
    }

    #[test]
    fn parse_args_test() {
        let mut cmd = command!{
            name: "test".to_owned(),
            args: [
                ["stringarg", StringArg, true, None],
                ["uintarg", UintArg, false, Some(1_u64)],
                ["intarg", IntArg, false, Some(1_i64)],
                ["floatarg", FloatArg, false, Some(1_f64)],
            ],
        };

        let raw_args = vec!["bad".to_owned()];
        let parsed = cmd.parse_args(raw_args);
        assert_eq!(parsed.is_err(), true);

        let raw_args = vec!["test".to_owned()];
        let parsed = cmd.parse_args(raw_args);
        assert_eq!(parsed.is_ok(), true);
        assert_eq!(parsed.unwrap().len(), 4);
    }
}