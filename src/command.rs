use clap::{Arg, ArgAction, ArgGroup, Command, ValueHint};
use lazy_static::lazy_static;

const AUTHORS: &str = "Timothy Cronin";
lazy_static! {
    static ref ABOUT: String = "
gret (global regular expression tree) is a command
line tool that searches a directory or file
for a matching regex expression and displays
matches in a tree.
"
    .to_string();
    static ref HELP: String = "{name}
by {{author}}
{{about}} 
{{usage}}

{{all-args}}"
        .to_string();
}

pub fn generate_command() -> Command {
    let mut command = Command::new("todo")
        .author(AUTHORS)
        .about(ABOUT.to_owned())
        .help_template(HELP.to_owned());

    command = add_expr_group(command);
    command = add_target_group(command);
    for opt in get_options() {
        command = command.arg(opt);
    }
    return command;
}

fn get_options() -> Vec<Arg> {
    vec![
        Arg::new("bland")
            .long("bland")
            .short('b')
            .value_name("bland")
            .help("if this option is present there will be no styling of text")
            .action(ArgAction::SetTrue),
        Arg::new("show_count")
            .long("show_count")
            .short('c')
            .value_name("Show Count")
            .help("if this option is present, display number of files matched in a directory and number of lines matched in a file")
            .action(ArgAction::SetTrue),
        Arg::new("search_hidden")
            .long("hidden")
            .short('a')
            .value_name("Search Hidden")
            .help("if this option is present gret will search hidden files")
            .action(ArgAction::SetTrue),
        Arg::new("max_depth")
            .long("max_depth")
            .short('m')
            .value_name("Max Depth")
            .help("the max depth the searcher will search")
            .action(ArgAction::Set),
        Arg::new("line_number")
        .long("line_number")
        .short('l')
        .value_name("Show Line Number")
        .help("if this option is present show the line number of the matched text")
        .action(ArgAction::SetTrue)
    ]
}

fn add_expr_group(mut command: Command) -> Command {
    let help = "specify the regex expression";
    let value_name = "Pattern";
    command = command.arg(
        Arg::new("expression_pos")
            .value_name(value_name)
            .help(help)
            .index(1),
    );

    command = command.arg(
        Arg::new("expression")
            .short('e')
            .value_name(value_name)
            .help(help)
            .action(ArgAction::Append),
    );

    command = command.group(
        ArgGroup::new("expression_group")
            .id("expressions")
            .args(["expression_pos", "expression"])
            .multiple(true)
            .required(true),
    );
    return command;
}

fn add_target_group(mut command: Command) -> Command {
    let help = "specify the search target. If none provided, search the current directory.";
    let value_name = "Target File or Directory";
    command = command.arg(
        Arg::new("target_pos")
            .value_name(value_name)
            .help(help)
            .value_hint(ValueHint::AnyPath)
            .index(2),
    );
    command = command.arg(
        Arg::new("target")
            .short('t')
            .long("target")
            .value_name(value_name)
            .help(help)
            .value_hint(ValueHint::AnyPath),
    );
    command = command.group(
        ArgGroup::new("target_group")
            .id("targets")
            .args(["target_pos", "target"]),
    );

    return command;
}
