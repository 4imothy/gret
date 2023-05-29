// SPDX-License-Identifier: Unlicense

// Colors add 60 to the number for bright
static RED_FG: &str = "\x1b[31m";
static GREEN_FG: &str = "\x1b[32m";
static YELLOW_FG: &str = "\x1b[33m";
static BLUE_FG: &str = "\x1b[34m";
static MAGENTA_FG: &str = "\x1b[35m";
static CYAN_FG: &str = "\x1b[36m";

pub static TODO_COLOR: &str = GREEN_FG;
pub static NOTE_COLOR: &str = MAGENTA_FG;
pub static HACK_COLOR: &str = YELLOW_FG;
pub static FIXME_COLOR: &str = RED_FG;

pub static FILE_COLOR: &str = CYAN_FG;
pub static DIR_COLOR: &str = BLUE_FG;

pub static ERROR_COLOR: &str = RED_FG;
pub static BIN_NAME_COLOR: &str = BLUE_FG;

// Other
pub static RESET: &str = "\x1b[0m";
pub static BOLD: &str = "\x1b[1m";
// Printing
pub static BRANCH_HAS_NEXT: &str = "├──";
pub static BRANCH_END: &str = "└──";
pub static VER_LINE_SPACER: &str = "│  ";
pub static SPACER: &str = "   ";
