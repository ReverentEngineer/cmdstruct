# cmdstruct

A lightweight macro for implementing commands with a struct

[![Documentation](https://docs.rs/cmdstruct/badge.svg)](https://docs.rs/cmdstruct)

## Usage

```rust
use cmdstruct::Command;

#[derive(Command)]
#[command(executable = "echo")]
struct Echo {

    /// Flag to provide
    #[arg(flag = "-n")]
    no_new_line: bool,

    /// String to echo
    #[arg]
    s: String

}

fn main() {
    let echo = Echo {
        no_new_line: true,
        s: "hello world".to_string()
    };

    echo.command().spawn();
}

```
