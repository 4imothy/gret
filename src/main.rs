// SPDX-License-Identifier: GPL-3.0

mod arg_parser;
mod errors;
mod formats;
mod ignores_parser;
mod printer;
mod searcher;
use arg_parser::parse_args;

fn main() -> std::io::Result<()> {
    let settings = parse_args(std::env::args().collect()).unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let depth: usize = 0;
    if settings.is_dir {
        let dir = searcher::start_search_dir(settings.path)?;
        printer::print_directory(dir, depth);
    } else {
        let file = searcher::search_singe_file(settings.path);
        match file {
            Some(f) => {
                printer::print_file(&f, depth);
            }
            _ => {}
        }
    }

    Ok(())
}
