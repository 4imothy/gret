// SPDX-License-Identifier: Unlicense

mod command;
mod formats;
mod menu;
mod printer;
use printer::write_results;
mod searcher;
use searcher::SearchedTypes;
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
            // only open the cli if there were matches
            if top_dir.borrow().children.len() > 0 || top_dir.borrow().found_files.len() > 0 {
                menu::draw(&mut out, SearchedTypes::Dir(top_dir)).unwrap_or_else(|e| {
                    exit_error(Errors::IOError {
                        cause: e.to_string(),
                    });
                });
            }
        } else {
            write_results(&mut out, &SearchedTypes::Dir(top_dir)).unwrap_or_else(|e| {
                exit_error(Errors::IOError {
                    cause: e.to_string(),
                })
            });
        }
    } else {
        let m_file = searcher::search_file(CONFIG.path.clone()).unwrap_or_else(|e| exit_error(e));
        if let Some(file) = m_file {
            let mut out = std::io::stdout().lock();
            if CONFIG.menu {
                // only open the cli if there were matches
                if file.lines.len() > 0 {
                    menu::draw(&mut out, SearchedTypes::File(file)).unwrap_or_else(|e| {
                        exit_error(Errors::IOError {
                            cause: e.to_string(),
                        });
                    });
                }
            } else {
                write_results(&mut out, &SearchedTypes::File(file)).unwrap_or_else(|e| {
                    exit_error(Errors::IOError {
                        cause: e.to_string(),
                    });
                });
            }
        }
    }
}

fn exit_error(e: Errors) -> ! {
    println!("{}", e);
    std::process::exit(1);
}
