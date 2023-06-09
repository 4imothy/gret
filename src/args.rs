// SPDX-License-Identifier: Unlicense

use crate::command::generate_command;
use crate::errors::Errors;
use crate::formats;
use atty::Stream;
use regex::bytes::Regex;
use std::path::PathBuf;

pub struct Config {
    pub path: PathBuf,
    pub is_dir: bool,
    pub styled: bool,
    pub patterns: Vec<Regex>,
    pub show_count: bool,
    pub search_hidden: bool,
    pub max_depth: Option<usize>,
    pub show_line_number: bool,
    pub menu: bool,
    pub terminator: String,
    pub reset: String,
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

    let show_count: bool = *matches.get_one::<bool>("show_count").unwrap();
    let search_hidden: bool = *matches.get_one::<bool>("search_hidden").unwrap();
    let show_line_number: bool = *matches.get_one::<bool>("line_number").unwrap();
    let menu: bool = *matches.get_one::<bool>("menu").unwrap();

    let max_depth_str: Option<&String> = matches.get_one::<String>("max_depth");

    let depth_result: Result<Option<usize>, Errors> = max_depth_str.map_or(Ok(None), |s| {
        s.parse::<usize>()
            .map(Some)
            .map_err(|_| Errors::StringToUsizeFail {
                cause: s.to_string(),
            })
    });
    let max_depth: Option<usize> = depth_result?;

    let target: Option<String> = matches
        .get_one::<String>("target_pos")
        .or_else(|| matches.get_one::<String>("target"))
        .map(|value| value.to_string());

    let path = if let Some(target) = target {
        let path = PathBuf::from(target);
        if !path.exists() {
            return Err(Errors::PathNotFound { cause: path });
        }
        path
    } else {
        std::env::current_dir().map_err(|_| Errors::FailedToGetCWD)?
    };

    let terminator = formats::get_terminator(menu);
    let reset = formats::get_reset(menu);

    Ok(Config {
        is_dir: path.is_dir(),
        path,
        styled,
        patterns,
        show_count,
        search_hidden,
        max_depth,
        show_line_number,
        menu,
        terminator,
        reset,
    })
}
