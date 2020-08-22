#[macro_export]
macro_rules! argument {
    ([
        $arg:expr,
        $desc:expr,
        $argtype:expr,
        $type:ty,
        $kind:expr,
        $default:expr
    ]) => {
        $crate::Arg::new(
            $arg,
            $desc,
            $argtype,
            std::any::type_name::<$type>(),
            $kind,
            $default,
        )
    };
}

#[macro_export]
macro_rules! command {
    (
        name: $name:expr,
        desc: $desc:expr,
        args: [
            $($arg:tt),* $(,)*
        ] $(,)*
    ) => {{
        let mut _cmd = $crate::Command::new($name, $desc);
        $(
            let arg = $crate::argument!($arg);
            _cmd.add_arg(arg);
        )*
        _cmd
    }};
}
