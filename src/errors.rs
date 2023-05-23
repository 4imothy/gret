// SPDX-License-Identifier: GPL-3.0

use crate::formats;
use std::fmt;
use std::path::PathBuf;

pub enum Errors {
    PathNotFound { cause: PathBuf },
    CantCanonicalize { cause: PathBuf },
    CantWrite,
    CantCollect { cause: String },
    UnknownArgument { cause: String },
    UnableToReadDir { cause: PathBuf },
}

fn error_prefix() -> String {
    format!(
        "{}{}Error: {}",
        formats::RED_FG,
        formats::BOLD,
        formats::RESET
    )
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_prefix = error_prefix();
        match &self {
            Errors::PathNotFound { cause } => {
                write!(
                    f,
                    "{}Path, {:?}, not found.",
                    error_prefix,
                    cause.as_os_str()
                )
            }
            Errors::UnknownArgument { cause } => {
                write!(f, "{}Unknown Argument, {}", error_prefix, cause)
            }
            Errors::CantWrite => {
                write!(f, "{}Can't print to Stdout", error_prefix)
            }
            Errors::CantCanonicalize { cause } => {
                write!(
                    f,
                    "{}Cannot get the absolute path of: {:?}",
                    error_prefix, cause
                )
            }
            Errors::CantCollect { cause } => {
                write!(f, "{}Can't collect items of type: {}", error_prefix, cause)
            }
            Errors::UnableToReadDir { cause } => {
                write!(f, "{}Unable to read directory: {:?}", error_prefix, cause)
            }
        }
    }
}
