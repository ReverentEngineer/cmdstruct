#![doc = include_str!("../README.md")]

pub use cmdstruct_macros::Command;

/// A trait representing a particular command
pub trait Command {

    /// Generate a spawnable command
    fn command(&self) -> std::process::Command;

}

/// A trait representing an argument to a command
pub trait Arg {

    /// Append as positional argument
    fn append_arg(&self, command: &mut std::process::Command);

    /// Append as an option
    fn append_option(&self, name: &str, command: &mut std::process::Command) {
        self.append_arg(command.arg(name));
    }
}

macro_rules! format_impl {
    ($($ty:ident) *) => {
        $(
        impl Arg for $ty {
            fn append_arg(&self, command: &mut std::process::Command)
            {
                command.arg(&format!("{}", self));
            }
        }
        )*
    }
}

format_impl!(u8 u16 u32 u64 usize);
format_impl!(i8 i16 i32 i64 isize);
format_impl!(char String);
format_impl!(f32 f64);

impl<T> Arg for Option<T>
where
    T: Arg,
{
    fn append_arg(&self, command: &mut std::process::Command) {
        match self {
            Some(arg) => arg.append_arg(command),
            None => {}
        }
    }

    fn append_option(&self, name: &str, command: &mut std::process::Command) {
        match self {
            Some(arg) => arg.append_option(name, command),
            None => {}
        }
    }
}

impl<T> Arg for Vec<T>
where
    T: Arg,
{
    fn append_arg(&self, command: &mut std::process::Command) {
        for argument in self {
            argument.append_arg(command);
        }
    }

    fn append_option(&self, name: &str, command: &mut std::process::Command) {
        for argument in self {
            argument.append_option(name, command);
        }
    }
}
