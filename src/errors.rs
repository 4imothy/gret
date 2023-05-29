// SPDX-License-Identifier: Unlicense

use formats::{BOLD, ERROR_COLOR, RESET};
use lazy_static::lazy_static;
use std::fmt;
use std::path::PathBuf;

lazy_static! {
    static ref ERROR_PREFIX: String = format!("{}{}Error: {}", ERROR_COLOR, BOLD, RESET);
}

pub enum Errors {
    PathNotFound { cause: PathBuf },
    CantWrite,
    CantGetName { cause: PathBuf },
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Errors::PathNotFound { cause } => {
                write!(
                    f,
                    "{}Path: `{}` was not found.",
                    ERROR_PREFIX.to_string(),
                    cause.to_string_lossy()
                )
            }
            Errors::CantWrite => {
                write!(f, "{}Can't print to Stdout", ERROR_PREFIX.to_string())
            }
            Errors::CantGetName { cause } => {
                write!(
                    f,
                    "{}Can't get the name of entry: `{}`",
                    ERROR_PREFIX.to_string(),
                    cause.to_string_lossy()
                )
            }
        }
    }
}
