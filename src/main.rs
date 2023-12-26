// SPDX-License-Identifier: CC-BY-4.0

// TODO put the line in the Selected struct around an os check
// TODO change the hidden option to still not searh .git directory i think or make that a seperate
// option, --hidden or -h to show . and -a or --all to search all files
// don't think there is a use case to search .git along with other files
// TODO fix completions
// TODO On highlighting for the menu had to overwrite the default
// fg to be white so that the background styling wouldn't
// disappear after a `RESET_COLOR` was called
// TODO Make a side bar for the menu that has numbers/letters corresponding with each row if one of those keys is pressed than enter that file
// TODO Make work for stdin, not sure how to work with branching

mod args;
mod command;
mod errors;
mod formats;
mod logger;
mod menu;
mod printer;
mod searcher;
use args::{parse_args, Config};
use errors::Errors;
use lazy_static::lazy_static;
use menu::Menu;
use printer::write_results;
use searcher::Searched;
use std::io::{stdout, StdoutLock};

lazy_static! {
    static ref CONFIG: Config = parse_args().unwrap_or_else(|e| {
        exit_error(e);
    });
}

fn main() {
    let mut out: StdoutLock = stdout().lock();
    if CONFIG.is_dir {
        let directories =
            searcher::search_dir(CONFIG.path.clone()).unwrap_or_else(|e| exit_error(e));
        if CONFIG.menu {
            // only open the cli if there were matches
            if directories.get(0).unwrap().children.len() > 0
                || directories.get(0).unwrap().files.len() > 0
            {
                start_menu(&mut out, Searched::Dir(directories));
            }
        } else {
            print_results(&mut out, Searched::Dir(directories));
        }
    } else {
        if let Some(file) =
            searcher::search_file(CONFIG.path.clone()).unwrap_or_else(|e| exit_error(e))
        {
            if CONFIG.menu {
                if file.lines.len() > 0 {
                    start_menu(&mut out, Searched::File(file));
                }
            } else {
                print_results(&mut out, Searched::File(file));
            }
        }
    }
}

fn start_menu(out: &mut StdoutLock, res: Searched) {
    Menu::draw(out, res).unwrap_or_else(|e| {
        exit_error(Errors::IOError {
            cause: e.to_string(),
        });
    });
}

fn print_results(out: &mut StdoutLock, searched: Searched) {
    write_results(out, &searched).unwrap_or_else(|e| {
        exit_error(Errors::IOError {
            cause: e.to_string(),
        })
    });
}

fn exit_error(e: Errors) -> ! {
    println!("{}", e);
    std::process::exit(1);
}
