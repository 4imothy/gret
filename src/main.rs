// SPDX-License-Identifier: Unlicense

mod printer;
mod searcher;
use printer::{print_single_file, start_print_directory};
mod arg_parser;
use arg_parser::parse_args;
mod errors;
use errors::Errors;
mod formats;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() {
    let settings = parse_args(std::env::args().collect()).unwrap_or_else(|e| {
        exit_error(e);
    });
    if settings.is_dir {
        let top_dir = searcher::begin_search_on_directory(settings.path).map_err(|e| exit_error(e));
        let mut out = std::io::stdout().lock();
        start_print_directory(&mut out, top_dir.unwrap()).unwrap_or_else(|e| exit_error(e));
    } else {
        let file = searcher::search_file(&settings.path).map_err(|e| exit_error(e));
        let mut out = std::io::stdout().lock();
        if let Ok(fi) = file {
            if let Some(f) = fi {
                print_single_file(&mut out, &f).unwrap_or_else(|e| exit_error(e));
            } else {
                // if there were no matches still print the name
                write_name(&mut out, settings.path);
            }
        }
    }
}

fn write_name(out: &mut io::StdoutLock, path: PathBuf) {
    if let Err(_) = writeln!(
        out,
        "{}",
        path.file_name().unwrap_or(OsStr::new("")).to_string_lossy()
    ) {
        exit_error(Errors::CantWrite);
    }
}

fn exit_error(e: Errors) -> ! {
    println!("{}", e);
    std::process::exit(1);
}
