extern crate redis_module;

use std::any::{Any, type_name};
use std::collections::HashMap;
use std::fmt::Debug;

use dyn_clonable::*;
use redis_module::{RedisError, parse_unsigned_integer, parse_integer, parse_float};

#[derive(Debug, PartialEq)]
pub struct Command {
    pub name: &'static str,
    required_args: Vec<Arg>,
    optional_args: Vec<Arg>,
    named_args: HashMap<&'static str, Arg>,
}

thread_local! {
    static TN_STRING: &'static str = type_name::<String>();
    static TN_U64: &'static str = type_name::<u64>();
    static TN_I64: &'static str = type_name::<i64>();
    static TN_F64: &'static str = type_name::<f64>();
}

macro_rules! parse_arg {
    (
        $arg:ident,
        $next_arg:ident
    ) => {
        match $arg.type_name {
            n if n == TN_STRING.with(|t| t.clone()) => {
                Box::new($next_arg.clone())
            },
            n if n == TN_U64.with(|t| t.clone()) => {
                Box::new(parse_unsigned_integer($next_arg.as_str())?)
            },
            n if n == TN_I64.with(|t| t.clone()) => {
                Box::new(parse_integer($next_arg.as_str())?)
            },
            n if n == TN_F64.with(|t| t.clone()) => {
                Box::new(parse_float($next_arg.as_str())?)
            },
            _ => return Err(RedisError::String(format!("{} is not a supported type", $arg.type_name)))
        }
    };
}

impl Command {
    pub fn new(name: &'static str) -> Self{
        Command {name, required_args: Vec::new(), optional_args: Vec::new(), named_args: HashMap::new()}
    }

    pub fn add_arg(&mut self, arg: Arg) {
        if arg.kwarg {
            self.named_args.insert(arg.arg, arg);
        } else {
            if arg.default.is_none() {
                self.required_args.push(arg);
            } else {
                self.optional_args.push(arg);
            }
        }
    }

    pub fn parse_args(&self, raw_args: Vec<String>) -> Result<HashMap<&'static str, Box<dyn Value>>, RedisError> {
        let mut raw_args = raw_args.into_iter();
        match raw_args.next() {
           Some(cmd_name) => {
               if cmd_name.to_lowercase() != self.name {
                   return Err(RedisError::String(format!("Expected {}, got {}", self.name, cmd_name)))
               }
           },
           None => return Err(RedisError::WrongArity)
        }
        
        let mut res = HashMap::new();

        // parse args
        let mut required_pos: usize = 0;
        let mut optional_pos: usize = 0;
        let mut do_optional = true;
        while let Some(next_arg) = raw_args.next() {
            // match required args
            if required_pos < self.required_args.len() {
                let arg = &self.required_args[required_pos];
                let val: Box<dyn Value> = parse_arg!(arg, next_arg);
                res.insert(arg.arg, val);
                required_pos += 1;
                continue;
            }
            
            if let Some(arg) = self.named_args.get(next_arg.to_lowercase().as_str()) {
                // if we can match named args, then done with optional
                if do_optional {
                    do_optional = false;
                }

                let val: Box<dyn Value> = match raw_args.next() {
                    Some(next) => parse_arg!(arg, next),
                    None => return Err(RedisError::WrongArity)
                };
                
                res.insert(arg.arg, val);
            } else {
                // match optional args
                if do_optional && optional_pos < self.optional_args.len() {
                    let arg = &self.optional_args[optional_pos];
                    let val: Box<dyn Value> = parse_arg!(arg, next_arg);
                    res.insert(arg.arg, val);
                    optional_pos += 1;
                } else {
                    return Err(RedisError::String(format!("Unexpected arg {}", next_arg)))
                }
            }
        }

        // check if all required args are fulfilled
        for v in self.required_args.iter() {
            if !res.contains_key(v.arg) {
                return Err(RedisError::String(format!("{} is required", v.arg)))
            }
        }

        // check if all optional args are fulfilled
        for v in self.optional_args.iter() {
            if !res.contains_key(v.arg) {
                res.insert(v.arg, v.default.as_ref().unwrap().clone());
            }
        }

        // check if all kwargs are fulfilled
        for (k, v) in self.named_args.iter() {
            if !res.contains_key(k) {
                if v.default.is_none() {
                    return Err(RedisError::String(format!("{} is required", v.arg)))
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
    pub kwarg: bool,
}

impl Arg {
    pub fn new(arg: &'static str, type_name: &'static str, default: Option<Box<dyn Value>>, kwarg: bool) -> Self {
        Arg {arg, type_name, default, kwarg}
    }
}

impl std::cmp::PartialEq for Arg {
    fn eq(&self, other: &Self) -> bool {
        self.arg == other.arg &&
        self.type_name == other.type_name &&
        self.default.is_none() == other.default.is_none() &&
        self.kwarg == other.kwarg
    }
}

#[macro_export]
macro_rules! argument {
    ([
        $arg:expr,
        $type:ty,
        $default:expr,
        $unnamed:expr
    ]) => {
        $crate::Arg::new($arg, std::any::type_name::<$type>(), $default, $unnamed)
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
        let mut _cmd = $crate::Command::new($name);
        $(
            let arg = argument!($arg);
            _cmd.add_arg(arg);
        )*
        _cmd
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
                ["stringarg", String, None, false],
                ["uintarg", u64, Some(Box::new(1_u64)), true],
                ["intarg", i64, Some(Box::new(1_i64)), true],
                ["floatarg", f64, Some(Box::new(1_f64)), true],
            ],
        };

        let mut exp = Command::new("test");
        let arg1 = Arg::new("stringarg", std::any::type_name::<String>(), None, false);
        let arg2 = Arg::new("uintarg", std::any::type_name::<u64>(), Some(Box::new(1_u64)), true);
        let arg3 = Arg::new("intarg", std::any::type_name::<i64>(), Some(Box::new(1_i64)), true);
        let arg4 = Arg::new("floatarg", std::any::type_name::<f64>(), Some(Box::new(1_f64)), true);
        exp.add_arg(arg1);
        exp.add_arg(arg2);
        exp.add_arg(arg3);
        exp.add_arg(arg4);

        assert_eq!(cmd, exp);
    }

    #[test]
    fn parse_args_test() {
        let cmd = command!{
            name: "test",
            args: [
                ["required", String, None, false],
                ["optional", String, Some(Box::new("foo".to_owned())), false],
                ["uintarg", u64, Some(Box::new(1_u64)), true],
                ["intarg", i64, None, true],
                ["floatarg", f64, None, true],
            ],
        };

        let raw_args = vec!["test".to_owned()];
        let parsed = cmd.parse_args(raw_args);
        assert_eq!(parsed.is_err(), true);

        let raw_args = vec![
            "test".to_owned(),
            "bar".to_owned(),
            "intarg".to_owned(),
            "2".to_owned(),
            "floatarg".to_owned(),
            "3.14".to_owned(),
        ];
        let parsed = cmd.parse_args(raw_args);
        assert_eq!(parsed.is_ok(), true);
        assert_eq!(parsed.is_err(), false);
        
        let mut parsed = parsed.unwrap();
        assert_eq!(
            parsed.remove("required").unwrap().as_string().unwrap(),
            "bar".to_owned()
        );
        assert_eq!(
            parsed.remove("optional").unwrap().as_string().unwrap(),
            "foo".to_owned()
        );
        assert_eq!(
            parsed.remove("uintarg").unwrap().as_u64().unwrap(),
            1_u64
        );
        assert_eq!(
            parsed.remove("intarg").unwrap().as_i64().unwrap(),
            2_i64
        );
        assert_eq!(
            parsed.remove("floatarg").unwrap().as_f64().unwrap(),
            3.14
        );
    }
}