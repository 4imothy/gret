// SPDX-License-Identifier: GPL-3.0

use crate::formats;
use lazy_static::lazy_static;
use std::fmt;
use std::path::PathBuf;

lazy_static! {
    static ref ERROR_PREFIX: String = format!(
        "{}{}Error: {}",
        formats::RED_FG,
        formats::BOLD,
        formats::RESET
    );
}

pub enum Errors {
    PathNotFound { cause: PathBuf },
    CantWrite,
    UnknownArgument { cause: String },
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
            Errors::UnknownArgument { cause } => {
                write!(
                    f,
                    "{}Unknown Argument, `{}`",
                    ERROR_PREFIX.to_string(),
                    cause
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
