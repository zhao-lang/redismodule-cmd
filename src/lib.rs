use std::any::{Any, TypeId};

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
}

#[derive(Debug)]
pub struct Arg {
    pub arg: String,
    pub optional: bool,
    pub argtype: TypeId,
    pub default: Option<Box<dyn Any>>,
}

impl Arg {
    pub fn new(argtype: TypeId, arg: String, optional: bool, default: Option<Box<dyn Any>>) -> Self {
        Arg {arg, optional, argtype, default}
    }
}

impl PartialEq for Arg {
    fn eq(&self, other: &Self) -> bool {
        if self.arg != other.arg {
            return false
        }

        if self.argtype != other.argtype {
            return false
        }

        if self.optional != other.optional {
            return false
        }

        if self.default.is_some() != other.default.is_some() {
            return false
        }

        if self.default.is_some() && other.default.is_some() {
            if self.default.as_ref().unwrap().type_id() != other.default.as_ref().unwrap().type_id() {
                return false
            }
        }

        true
    }
}

#[macro_export]
macro_rules! command {
    (
        name: $name:expr,
        args: [
            $([
                $argtype:ty,
                $arg:expr,
                $optional:expr,
                $default:expr
            ]),* $(,)*
        ] $(,)*
    ) => {{
        use std::any::TypeId;
        let mut cmd = $crate::Command::new($name);
        $(
            cmd.add_arg($crate::Arg::new(TypeId::of::<$argtype>(), $arg, $optional, $default));
        )*
        cmd
    }};
}

#[cfg(test)]
mod tests {
    use super::{Arg, Command};
    use std::any::TypeId;

    #[test]
    fn macro_test() {
        let cmd = command!{
            name: "test".to_owned(),
            args: [
                [String, "stringarg".to_owned(), true, None]
            ],
        };

        let mut exp = Command::new("test".to_owned());
        let arg = Arg::new(TypeId::of::<String>(), "stringarg".to_owned(), true, None);
        exp.add_arg(arg);

        assert_eq!(cmd, exp);
    }
}