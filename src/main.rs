// SPDX-License-Identifier: Unlicense

mod command;
mod formats;
mod menu;
mod printer;
mod searcher;
use printer::{print_single_file, start_print_directory};
mod args;
use args::{parse_args, Config};
mod errors;
use errors::Errors;
use lazy_static::lazy_static;

lazy_static! {
    static ref CONFIG: Config = parse_args().unwrap_or_else(|e| {
        exit_error(e);
    });
}

fn main() {
    if CONFIG.is_dir {
        let top_dir = searcher::begin_search_on_directory(CONFIG.path.clone())
            .unwrap_or_else(|e| exit_error(e));
        let mut out = std::io::stdout().lock();
        if CONFIG.menu {
            menu::draw(&mut out, top_dir).unwrap_or_else(|_| exit_error(Errors::CantWrite));
        } else {
            start_print_directory(&mut out, &top_dir)
                .unwrap_or_else(|_| exit_error(Errors::CantWrite));
        }
    } else {
        let mut out = std::io::stdout().lock();
        let file = searcher::search_file(CONFIG.path.clone()).unwrap_or_else(|e| exit_error(e));
        print_single_file(&mut out, &file).unwrap_or_else(|_| exit_error(Errors::CantWrite));
    }
}

fn exit_error(e: Errors) -> ! {
    println!("{}", e);
    std::process::exit(1);
}
