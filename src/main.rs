// SPDX-License-Identifier: GPL-3.0

mod arg_parser;
mod errors;
mod formats;
mod ignores_parser;
mod searcher;
use arg_parser::parse_args;

fn main() -> std::io::Result<()> {
    let settings = parse_args(std::env::args().collect()).unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    searcher::search(settings.path, settings.is_dir)?;

    Ok(())
}
