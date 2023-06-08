// SPDX-License-Identifier: Unlicense

use crossterm::style::{Attribute, Color, ContentStyle, StyledContent, Stylize};
use std::fmt::Display;

const FG_RED: &str = "\x1b[31m";
const FG_GREEN: &str = "\x1b[32m";
const FG_YELLOW: &str = "\x1b[33m";
const FG_BLUE: &str = "\x1b[34m";
const FG_MAGENTA: &str = "\x1b[35m";
const FG_CYAN: &str = "\x1b[36m";

pub const FILE_COLOR: &str = FG_CYAN;
pub const DIR_COLOR: &str = FG_BLUE;

pub const ERROR_COLOR: &str = FG_RED;

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

pub const BRANCH_HAS_NEXT: &str = "├──";
pub const BRANCH_END: &str = "└──";
pub const VER_LINE_SPACER: &str = "│  ";
pub const SPACER: &str = "   ";
pub const LINE_NUMBER_COLOR: &str = FG_YELLOW;

const MATCHED_COLORS: [&str; 3] = [FG_GREEN, FG_MAGENTA, FG_RED];

pub fn get_color(i: usize) -> Vec<u8> {
    MATCHED_COLORS[i % MATCHED_COLORS.len()].as_bytes().to_vec()
}

pub fn dir_name(name: &str) -> StyledContent<&str> {
    name.with(Color::Blue).attribute(Attribute::Bold)
}

pub fn file_name(name: &str) -> StyledContent<&str> {
    name.with(Color::Cyan).attribute(Attribute::Bold)
}

// trait StyledContentExt<D: Display> {
//     fn dir_name(&self) -> StyledContent<D>;
// }

// impl<D: Display> StyledContentExt<D> for StyledContent<D> {
//     fn dir_name(&self) -> StyledContent<D> {
//         self.style = {}
//     }
// }
