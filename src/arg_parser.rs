// SPDX-License-Identifier: GPL-3.0

use crate::errors::Errors;
use std::path::PathBuf;

pub struct Settings {
    pub path: PathBuf,
    pub is_dir: bool,
}

impl Settings {
    fn new() -> Self {
        Settings {
            path: std::env::current_dir().expect("Failed to retrieve current directory"),
            is_dir: true,
        }
    }
}

pub fn parse_args(args: Vec<String>) -> Result<Settings, Errors> {
    let mut settings = Settings::new();
    // 1 is used to store relative bin path
    if args.len() < 2 {}

    for arg in args.iter().skip(1) {
        match arg {
            // TODO match other options options
            _ => {
                // either an unknown or a path to search
                let path = PathBuf::from(arg);
                if !path.exists() {
                    if arg.chars().nth(0) == Some('-') {
                        return Err(Errors::UnknownArgument {
                            cause: arg.to_string(),
                        });
                    }
                    return Err(Errors::PathNotFound { cause: path });
                }
                settings.is_dir = path.is_dir();
                settings.path = path;
            }
        }
    }

    return Ok(settings);
}
