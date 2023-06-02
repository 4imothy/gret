// SPDX-License-Identifier: Unlicense

mod command;
mod printer;
mod searcher;
use printer::{print_single_file, start_print_directory};
mod args;
use args::parse_args;
mod errors;
use errors::Errors;

fn main() {
    let config = parse_args().unwrap_or_else(|e| {
        exit_error(e);
    });
    if config.is_dir {
        let top_dir = searcher::begin_search_on_directory(&config).map_err(|e| exit_error(e));
        let mut out = std::io::stdout().lock();
        start_print_directory(&mut out, top_dir.unwrap(), config.styled)
            .unwrap_or_else(|e| exit_error(e));
    } else {
        let mut out = std::io::stdout().lock();
        let file = searcher::search_file(&config.path, &config).map_err(|e| exit_error(e));
        if let Ok(fi) = file {
            print_single_file(&mut out, &fi, config.styled).unwrap_or_else(|e| exit_error(e));
        }
    }
}

fn exit_error(e: Errors) -> ! {
    println!("{}", e);
    std::process::exit(1);
}
