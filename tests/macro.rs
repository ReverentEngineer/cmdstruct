extern crate cmdstruct;

use cmdstruct::command;

#[test]
fn option() {
    #[command(executable = "test")]
    struct Test {
        
        #[arg(option = "--input")]
        a: String 

    }

    let test = Test {
        a: "a".to_string()
    };

    let command = test.command();
    assert_eq!(command.get_args().collect::<Vec<_>>(), vec!["--input", "a"]);
    assert_eq!(command.get_program(), "test");
}

#[test]
fn positional() {
    #[command(executable = "test")]
    struct Test {
        
        #[arg]
        a: String 

    }

    let test = Test {
        a: "a".to_string()
    };

    let command = test.command();
    assert_eq!(command.get_args().collect::<Vec<_>>(), vec!["a"]);
    assert_eq!(command.get_program(), "test");
}

#[test]
fn flag() {
    #[command(executable = "test")]
    struct Test {
        
        #[arg(flag = "-a")]
        a: bool

    }

    let test = Test {
        a: true
    };

    let command = test.command();
    assert_eq!(command.get_args().collect::<Vec<_>>(), vec!["-a"]);
    assert_eq!(command.get_program(), "test");
}
