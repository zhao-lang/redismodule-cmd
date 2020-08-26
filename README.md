# redismodule-cmd

Command parser for redis modules with the ability to
auto-generate command references

## Usage

### Creating a Command

The `command` macro generates a `Command` instance, which then can be used
to parse redis command args.

```rust
let cmd = command!{
    name: "command name",
    desc: "command description",
    args: [
        [
            "arg name",
            "arg description",
            ArgType::Arg|Kwarg,         // plain arg or keyword arg
            String|u64|i64|f64,         // data type
            Collection::Unit|Vec,       // whether to expect a vec of inputs
            Option<Box<default_value>>  // default value
        ],
        ...
    ],
}

let mut parsed = cmd.parse_args(args).unwrap();
let input = parsed.remove("arg name").unwrap().as_string|as_u64|as_i64|as_f64().unwrap();
```

### Auto-generation of Command Reference

The `rediscmd_doc` attribute marks a command for auto-generate of command reference.
The command reference is generated on build to `./doc/COMMAND_REFERENCE_GEN.md`.
The attribute takes an optional arg, e.g. `#[rediscmd_doc(clean)]` to clear the file
before generating command reference.

```rust
#[rediscmd_doc]
let cmd = command!{
    name: "command name",
    desc: "command description",
    args: [
        ...
    ],
}
```

## Examples

see [lib/examples/](lib/examples/)
