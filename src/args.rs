// SPDX-License-Identifier: Unlicense

use crate::command::generate_command;
use crate::errors::Errors;
use atty::Stream;
use regex::bytes::Regex;
use std::path::PathBuf;

pub struct Config {
    pub path: PathBuf,
    pub styled: bool,
    pub patterns: Vec<Regex>,
    pub is_dir: bool,
}

pub fn parse_args() -> Result<Config, Errors> {
    let matches = generate_command().get_matches();
    let mut patterns: Vec<Regex> = Vec::new();

    if let Some(expr) = matches.get_one::<String>("expression_pos") {
        patterns.push(Regex::new(expr).map_err(|_| {
            return Errors::InvalidRegex {
                cause: expr.to_string(),
            };
        })?);
    }
    if let Some(exprs) = matches.get_many::<String>("expression") {
        for e in exprs.into_iter() {
            patterns.push(Regex::new(e).map_err(|_| {
                return Errors::InvalidRegex {
                    cause: e.to_string(),
                };
            })?);
        }
    }

    let styled = if *matches.get_one::<bool>("bland").unwrap_or(&false) || !atty::is(Stream::Stdout)
    {
        false
    } else {
        true
    };

    let target: Option<String> = matches
        .get_one::<String>("target_pos")
        .or_else(|| matches.get_one::<String>("target"))
        .map(|value| value.to_string());

    if let Some(target) = target {
        let path = PathBuf::from(target);
        if !path.exists() {
            return Err(Errors::PathNotFound { cause: path });
        }
        return Ok(Config {
            is_dir: path.is_dir(),
            styled,
            patterns,
            path,
        });
    } else {
        return Ok(Config {
            path: std::env::current_dir().map_err(|_| {
                return Errors::FailedToGetCWD;
            })?,
            styled,
            patterns,
            is_dir: true,
        });
    }
}
