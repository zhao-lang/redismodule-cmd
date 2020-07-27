extern crate redis_module;

use std::any::{Any, type_name};
use std::collections::HashMap;
use std::fmt::Debug;
use redis_module::{RedisError};

#[derive(Debug, PartialEq)]
pub struct Command {
    pub name: String,
    args: Vec<Arg>,
}

impl Command {
    pub fn new(name: String) -> Self{
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

        // parse args
        let next_arg = raw_args.next();
        for arg in self.args.iter() {


        }

        Ok(res)
    }
}


#[derive(Debug)]
pub struct Arg {
    pub arg: String,
    pub type_name: &'static str,
    pub optional: bool,
    pub default: Option<Box<dyn Any>>,
}

impl Arg {
    pub fn new(arg: String, type_name: &'static str, optional: bool, default: Option<Box<dyn Any>>) -> Self {
        Arg {arg, type_name, optional, default}
    }
}

impl std::cmp::PartialEq for Arg {
    fn eq(&self, other: &Self) -> bool {
        self.arg == other.arg &&
        self.type_name == other.type_name &&
        self.optional == other.optional
    }
}

#[macro_export]
macro_rules! argument {
    ([
        $arg:expr,
        $type:ty,
        $optional:expr,
        $default:expr
    ]) => {
        $crate::Arg::new($arg.to_owned(), std::any::type_name::<$type>(), $optional, $default)
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
    use super::{Arg, Command};

    #[test]
    fn macro_test() {
        let cmd = command!{
            name: "test".to_owned(),
            args: [
                ["stringarg", String, true, None],
                ["uintarg", u64, false, Some(Box::new(1_u64))],
                ["intarg", i64, false, Some(Box::new(1_i64))],
                ["floatarg", f64, false, Some(Box::new(1_f64))],
            ],
        };

        let mut exp = Command::new("test".to_owned());
        let arg1 = Arg::new("stringarg".to_owned(), std::any::type_name::<String>(), true, None);
        let arg2 = Arg::new("uintarg".to_owned(), std::any::type_name::<u64>(), false, Some(Box::new(1_u64)));
        let arg3 = Arg::new("intarg".to_owned(), std::any::type_name::<i64>(), false,Some(Box::new(1_i64)));
        let arg4 = Arg::new("floatarg".to_owned(), std::any::type_name::<f64>(), false,Some(Box::new(1_f64)));
        exp.add_arg(arg1);
        exp.add_arg(arg2);
        exp.add_arg(arg3);
        exp.add_arg(arg4);

        assert_eq!(cmd, exp);
    }

    #[test]
    fn parse_args_test() {
        let mut cmd = command!{
            name: "test".to_owned(),
            args: [
                ["stringarg", String, true, None],
                ["uintarg", u64, false, Some(Box::new(1_u64))],
                ["intarg", i64, false, Some(Box::new(1_i64))],
                ["floatarg", f64, false, Some(Box::new(1_f64))],
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