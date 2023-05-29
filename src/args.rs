// SPDX-License-Identifier: Unlicense

use crate::command::generate_command;
use crate::errors::Errors;
use std::path::PathBuf;

pub struct Config {
    pub path: PathBuf,
    pub is_dir: bool,
}

pub fn parse_args() -> Result<Config, Errors> {
    let matches = generate_command().get_matches();

    if let Some(dir) = matches.get_one::<String>("directory") {
        let path = PathBuf::from(dir);
        if !path.exists() {
            return Err(Errors::PathNotFound { cause: path });
        }
        return Ok(Config {
            is_dir: path.is_dir(),
            path,
        });
    } else {
        return Ok(Config {
            path: std::env::current_dir().expect("Failed to retrieve current directory"),
            is_dir: true,
        });
    };
}
