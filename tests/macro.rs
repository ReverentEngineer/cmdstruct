extern crate cmdstruct;

use cmdstruct::Command;

#[test]
fn option() {
    #[derive(Command)]
    #[command(executable = "test")]
    struct Test {
        #[arg(option = "--input")]
        a: String,
    }

    let test = Test { a: "a".to_string() };

    let command = test.command();
    assert_eq!(command.get_args().collect::<Vec<_>>(), vec!["--input", "a"]);
    assert_eq!(command.get_program(), "test");
}

#[test]
fn option_optional() {
    #[derive(Command)]
    #[command(executable = "test")]
    struct Test {
        #[arg(option = "--input")]
        a: Option<usize>,
    }

    let mut test = Test { a: Some(0) };

    let command = test.command();
    assert_eq!(command.get_args().collect::<Vec<_>>(), vec!["--input", "0"]);
    assert_eq!(command.get_program(), "test");
    test.a = None;
    let command = test.command();
    assert_eq!(
        command.get_args().collect::<Vec<&std::ffi::OsStr>>(),
        Vec::<&std::ffi::OsStr>::new()
    );
    assert_eq!(command.get_program(), "test");
}

#[test]
fn option_int() {
    #[derive(Command)]
    #[command(executable = "test")]
    struct Test {
        #[arg(option = "--input")]
        a: usize,
    }

    let test = Test { a: 3 };

    let command = test.command();
    assert_eq!(command.get_args().collect::<Vec<_>>(), vec!["--input", "3"]);
    assert_eq!(command.get_program(), "test");
}

#[test]
fn positional() {
    #[derive(Command)]
    #[command(executable = "test")]
    struct Test {
        #[arg]
        a: String,
    }

    let test = Test { a: "a".to_string() };

    let command = test.command();
    assert_eq!(command.get_args().collect::<Vec<_>>(), vec!["a"]);
    assert_eq!(command.get_program(), "test");
}

#[test]
fn positional_usize() {
    #[derive(Command)]
    #[command(executable = "test")]
    struct Test {
        #[arg]
        a: usize,
    }

    let test = Test { a: 0 };

    let command = test.command();
    assert_eq!(command.get_args().collect::<Vec<_>>(), vec!["0"]);
    assert_eq!(command.get_program(), "test");
}

#[test]
fn flag() {
    #[derive(Command)]
    #[command(executable = "test")]
    struct Test {
        #[arg(flag = "-a")]
        a: bool,
    }

    let test = Test { a: true };

    let command = test.command();
    assert_eq!(command.get_args().collect::<Vec<_>>(), vec!["-a"]);
    assert_eq!(command.get_program(), "test");
}

#[test]
fn executable_fn() {
    fn exe(test: &Test) -> String {
        format!("test-{}", test.suffix)
    }

    #[derive(Command)]
    #[command(executable_fn = exe)]
    struct Test {
        suffix: String,
    }

    let test = Test {
        suffix: "abc".to_string(),
    };

    let command = test.command();
    assert_eq!(command.get_program(), "test-abc");
}
