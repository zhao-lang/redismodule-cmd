extern crate redis_module;

use std::any::{Any, type_name};
use std::collections::HashMap;
use std::fmt::Debug;

use dyn_clonable::*;
use redis_module::{RedisError, NextArg};

#[derive(Debug, PartialEq)]
pub struct Command {
    pub name: &'static str,
    args: HashMap<&'static str, Arg>,
}

// // type_name is not yet a stable const fn
// const tn_string: &'static str = type_name::<String>();
// const tn_u64: &'static str = type_name::<u64>();
// const tn_i64: &'static str = type_name::<i64>();
// const tn_f64: &'static str = type_name::<f64>();

impl Command {
    pub fn new(name: &'static str) -> Self{
        Command {name, args: HashMap::new()}
    }

    pub fn add_arg(&mut self, arg: Arg) {
        self.args.insert(arg.arg, arg);
    }

    pub fn parse_args(&mut self, raw_args: Vec<String>) -> Result<HashMap<&'static str, Box<dyn Value>>, RedisError> {
        let mut raw_args = raw_args.into_iter();
        match raw_args.next() {
           Some(cmd_name) => {
               if cmd_name != self.name {
                   return Err(RedisError::String(format!("Expected {}, got {}", self.name, cmd_name)))
               }
           },
           None => return Err(RedisError::WrongArity)
        }
        
        let mut res = HashMap::new();

        // here until type_name is a stable const fn
        let tn_string = type_name::<String>();
        let tn_u64 = type_name::<u64>();
        let tn_i64 = type_name::<i64>();
        let tn_f64 = type_name::<f64>();

        // parse args
        while let Some(next_arg) = raw_args.next() {
            if let Some(arg) = self.args.get(next_arg.as_str()) {
                // parse arg
                let val: Box<dyn Value> = match arg.type_name {
                    n if n == tn_string => {
                        Box::new(raw_args.next_string()?)
                    },
                    n if n == tn_u64 => {
                        Box::new(raw_args.next_u64()?)
                    },
                    n if n == tn_i64 => {
                        Box::new(raw_args.next_i64()?)
                    },
                    n if n == tn_f64 => {
                        Box::new(raw_args.next_f64()?)
                    },
                    _ => return Err(RedisError::String(format!("{} is not a supported type", arg.type_name)))
                };
                
                res.insert(arg.arg, val);
            } else {
                return Err(RedisError::String(format!("Unexpected arg {}", next_arg)))
            }
        }
        raw_args.done()?;

        // check if all args are fulfilled
        for (k, v) in self.args.iter() {
            if !res.contains_key(k) && v.default.is_none() {
                return Err(RedisError::String(format!("{} is required", k)))
            }

            if !res.contains_key(k) && v.default.is_some() {
                if v.default.is_none() {
                    return Err(RedisError::String(format!("{} is has no default value", v.arg)))
                }
                res.insert(k.to_owned(), v.default.as_ref().unwrap().clone());
            }
        }

        Ok(res)
    }
}

#[clonable]
pub trait Value: Any + Debug + Clone {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_string(self: Box<Self>) -> Result<String, Box<dyn Any>>;
    fn as_u64(self: Box<Self>) -> Result<u64, Box<dyn Any>>;
    fn as_i64(self: Box<Self>) -> Result<i64, Box<dyn Any>>;
    fn as_f64(self: Box<Self>) -> Result<f64, Box<dyn Any>>;
}

impl<T: Any + Debug + Clone > Value for T {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }

    fn as_string(self: Box<Self>) -> Result<String, Box<dyn Any>> {
        match self.into_any().downcast::<String>() {
            Ok(d) => Ok(*d),
            Err(e) => Err(e)
        }
    }

    fn as_u64(self: Box<Self>) -> Result<u64, Box<dyn Any>> {
        match self.into_any().downcast::<u64>() {
            Ok(d) => Ok(*d),
            Err(e) => Err(e)
        }
    }

    fn as_i64(self: Box<Self>) -> Result<i64, Box<dyn Any>> {
        match self.into_any().downcast::<i64>() {
            Ok(d) => Ok(*d),
            Err(e) => Err(e)
        }
    }

    fn as_f64(self: Box<Self>) -> Result<f64, Box<dyn Any>> {
        match self.into_any().downcast::<f64>() {
            Ok(d) => Ok(*d),
            Err(e) => Err(e)
        }
    }
}

#[derive(Debug)]
pub struct Arg {
    pub arg: &'static str,
    pub type_name: &'static str,
    pub default: Option<Box<dyn Value>>,
}

impl Arg {
    pub fn new(arg: &'static str, type_name: &'static str, default: Option<Box<dyn Value>>) -> Self {
        Arg {arg, type_name, default}
    }
}

impl std::cmp::PartialEq for Arg {
    fn eq(&self, other: &Self) -> bool {
        self.arg == other.arg &&
        self.type_name == other.type_name &&
        self.default.is_none() == other.default.is_none()
    }
}

#[macro_export]
macro_rules! argument {
    ([
        $arg:expr,
        $type:ty,
        $default:expr
    ]) => {
        $crate::Arg::new($arg, std::any::type_name::<$type>(), $default)
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
            name: "test",
            args: [
                ["stringarg", String, None],
                ["uintarg", u64, Some(Box::new(1_u64))],
                ["intarg", i64, Some(Box::new(1_i64))],
                ["floatarg", f64, Some(Box::new(1_f64))],
            ],
        };

        let mut exp = Command::new("test");
        let arg1 = Arg::new("stringarg", std::any::type_name::<String>(), None);
        let arg2 = Arg::new("uintarg", std::any::type_name::<u64>(), Some(Box::new(1_u64)));
        let arg3 = Arg::new("intarg", std::any::type_name::<i64>(), Some(Box::new(1_i64)));
        let arg4 = Arg::new("floatarg", std::any::type_name::<f64>(), Some(Box::new(1_f64)));
        exp.add_arg(arg1);
        exp.add_arg(arg2);
        exp.add_arg(arg3);
        exp.add_arg(arg4);

        assert_eq!(cmd, exp);
    }

    #[test]
    fn parse_args_test() {
        let mut cmd = command!{
            name: "test",
            args: [
                ["stringarg", String, Some(Box::new("foo".to_owned()))],
                ["uintarg", u64, Some(Box::new(1_u64))],
                ["intarg", i64, None],
                ["floatarg", f64, None],
            ],
        };

        let raw_args = vec!["bad".to_owned()];
        let parsed = cmd.parse_args(raw_args);
        assert_eq!(parsed.is_err(), true);

        let raw_args = vec![
            "test".to_owned(),
            "stringarg".to_owned(),
            "bar".to_owned(),
            "intarg".to_owned(),
            "2".to_owned(),
            "floatarg".to_owned(),
            "3.14".to_owned(),
        ];
        let parsed = cmd.parse_args(raw_args);
        assert_eq!(parsed.is_ok(), true);
        assert_eq!(parsed.is_err(), false);
        
        let parsed = parsed.unwrap();
        assert_eq!(
            parsed.get("stringarg").unwrap().clone().as_string().unwrap(),
            "bar".to_owned()
        );
        assert_eq!(
            parsed.get("uintarg").unwrap().clone().as_u64().unwrap(),
            1_u64
        );
        assert_eq!(
            parsed.get("intarg").unwrap().clone().as_i64().unwrap(),
            2_i64
        );
        assert_eq!(
            parsed.get("floatarg").unwrap().clone().as_f64().unwrap(),
            3.14
        );
    }
}