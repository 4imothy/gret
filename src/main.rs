// SPDX-License-Identifier: GPL-3.0

mod arg_parser;
mod errors;
use errors::Errors;
mod formats;
mod ignores_parser;
mod printer;
mod searcher;
use arg_parser::parse_args;

fn main() {
    let settings = parse_args(std::env::args().collect()).unwrap_or_else(|e| {
        exit_error(e);
    });

    if settings.is_dir {
        let dir = searcher::start_search_dir(settings.path).unwrap_or_else(|e| exit_error(e));

        let mut out = std::io::stdout().lock();
        // printer::print_directory(&mut out, dir, depth, "".to_string(), true)?;
        printer::start_print_directory(&mut out, dir).unwrap_or_else(|e| exit_error(e));
    } else {
        // this returns none if the file in non-UTF-8
        let file = searcher::search_singe_file(settings.path);
        match file {
            Some(f) => {
                let mut out = std::io::stdout().lock();
                printer::print_single_file(&mut out, &f).unwrap_or_else(|e| {
                    exit_error(e);
                });
            }
            _ => {}
        }
    }
}

fn exit_error(e: Errors) -> ! {
    println!("{}", e);
    std::process::exit(1);
}
