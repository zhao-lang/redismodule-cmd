#[macro_use]
extern crate redis_module;

#[macro_use]
extern crate redismodule_cmd;

use redis_module::{Context, RedisError, RedisResult};
use redismodule_cmd::Command;

thread_local! {
    static CMD: Command = command!{
        name: "hello.foo",
        args: [
            ["input", String, None, false],
            ["optional", String, Some(Box::new("baz".to_owned())), false],
            ["n", u64, Some(Box::new(1_u64)), true],
        ],
    };
}

fn hello_foo(_: &Context, args: Vec<String>) -> RedisResult {
    let mut parsed = CMD.with(|cmd| {
        cmd.parse_args(args)
    })?;

    let input = parsed.remove("input").unwrap().as_string().unwrap();
    let opt = parsed.remove("optional").unwrap().as_string().unwrap();
    let n = parsed.remove("n").unwrap().as_u64().unwrap();

    let mut response = vec![input; n as usize];
    response.push(opt);

    return Ok(response.into());
}

//////////////////////////////////////////////////////

redis_module! {
    name: "hello",
    version: 1,
    data_types: [],
    commands: [
        ["hello.foo", hello_foo, "", 0, 0, 0],
    ],
}

//////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use redis_module::RedisValue;

    fn run_hello_foo(args: &[&str]) -> RedisResult {
        hello_foo(
            &Context::dummy(),
            args.iter().map(|v| String::from(*v)).collect(),
        )
    }

    #[test]
    fn hello_foo_valid_args() {
        let result = run_hello_foo(&vec!["hello.foo", "bar", "n", "2"]);

        match result {
            Ok(RedisValue::Array(v)) => {
                let exp = vec!["bar".to_owned(), "bar".to_owned(), "baz".to_owned()];
                assert_eq!(
                    v,
                    exp
                        .into_iter()
                        .map(RedisValue::BulkString)
                        .collect::<Vec<_>>()
                );
            }
            _ => assert!(false, "Bad result: {:?}", result),
        }
    }

    #[test]
    fn hello_foo_invalid_args() {
        let result = run_hello_foo(&vec!["hello.foo", "n", "2", "3"]);

        match result {
            Err(RedisError::String(s)) => {
                assert_eq!(s, "Unexpected arg 3");
            }
            _ => assert!(false, "Bad result: {:?}", result),
        }
    }

    #[test]
    fn hello_foo_invalid_cmd() {
        let result = run_hello_foo(&vec!["hello", "n", "2"]);

        match result {
            Err(RedisError::String(s)) => {
                assert_eq!(s, "Expected hello.foo, got hello");
            }
            _ => assert!(false, "Bad result: {:?}", result),
        }

        let result = run_hello_foo(&vec!["hello.foo", "bar", "n", "2", "bad"]);

        match result {
            Err(RedisError::String(s)) => {
                assert_eq!(s, "Unexpected arg bad");
            }
            _ => assert!(false, "Bad result: {:?}", result),
        }
    }
}