use clap::{Arg, Command, ValueHint};
use formats::{BIN_NAME_COLOR, BOLD, RESET, TODO_COLOR};
use lazy_static::lazy_static;

const AUTHORS: &str = "Timothy Cronin";
lazy_static! {
    static ref ABOUT: String = format!(
        "
todo is a command line tool for code analysis.
It searches directories, highlighting {}TODO{},
{}HACK{}, {}FIXME{}, and {}NOTE{} instances.
",
        TODO_COLOR, RESET, TODO_COLOR, RESET, TODO_COLOR, RESET, TODO_COLOR, RESET
    );
    static ref HELP: String = {
        format!(
            "{}{}{{name}}{}\nby {{author}}\n{{about}}\n{{all-args}}",
            BIN_NAME_COLOR, BOLD, RESET
        )
    };
}

pub fn generate_command() -> Command {
    let mut command = Command::new("todo")
        .author(AUTHORS)
        .about(ABOUT.to_owned())
        .help_template(HELP.to_owned());

    for arg in get_args() {
        command = command.arg(arg);
    }
    return command;
}

fn get_args() -> Vec<Arg> {
    vec![Arg::new("directory")
        .value_name("DIRECTORY")
        .help("Specify the search target. If none provided, search the current directory.")
        .index(1)
        .value_hint(ValueHint::AnyPath)]
}
