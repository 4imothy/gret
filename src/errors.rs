// SPDX-License-Identifier: Unlicense

use crate::formats::error_prefix;
use crossterm::ErrorKind;
use std::fmt;
use std::path::PathBuf;

pub enum Errors {
    PathNotFound { cause: PathBuf },
    CantWrite,
    CantGetName { cause: PathBuf },
    InvalidRegex { cause: String },
    FailedToGetCWD,
    StringToUsizeFail { cause: String },
    CrosstermError { cause: ErrorKind },
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_prefix: String = error_prefix();
        match &self {
            Errors::PathNotFound { cause } => {
                write!(
                    f,
                    "{}Path: `{}` was not found.",
                    error_prefix,
                    cause.display()
                )
            }
            Errors::CantWrite => {
                write!(f, "{}Can't print to Stdout", error_prefix)
            }
            Errors::CantGetName { cause } => {
                write!(
                    f,
                    "{}Can't get the name of entry: `{}`",
                    error_prefix,
                    cause.display()
                )
            }
            Errors::InvalidRegex { cause } => {
                write!(f, "{}Invalid Regex Pattern: `{}`", error_prefix, cause)
            }
            Errors::FailedToGetCWD => {
                write!(f, "{}Failed to get the current directory", error_prefix,)
            }
            Errors::StringToUsizeFail { cause } => {
                write!(
                    f,
                    "{}Failed to parse `{}` to an unsigned integer",
                    error_prefix, cause,
                )
            }
            Errors::CrosstermError { cause } => {
                write!(
                    f,
                    "{} Error styling terminal with crossterm: {}",
                    error_prefix, cause
                )
            }
        }
    }
}
