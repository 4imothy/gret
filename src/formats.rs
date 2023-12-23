// SPDX-License-Identifier: CC-BY-4.0

use crossterm::style::{
    Attribute, Color, ResetColor, SetAttribute, SetForegroundColor, StyledContent, Stylize,
};

pub const MENU_SELECTED: Color = Color::DarkGrey;

pub const RESET: SetAttribute = SetAttribute(Attribute::Reset);
pub const RESET_COLOR: ResetColor = ResetColor;
pub const NO_BOLD: SetAttribute = SetAttribute(Attribute::NormalIntensity);
pub const BOLD: SetAttribute = SetAttribute(Attribute::Bold);
const RED_FG: SetForegroundColor = SetForegroundColor(Color::Red);
const GREEN_FG: SetForegroundColor = SetForegroundColor(Color::Green);
pub const LINE_NUMBER_FG: SetForegroundColor = SetForegroundColor(Color::Yellow);
pub const DEFAULT_FG: SetForegroundColor = SetForegroundColor(Color::White);
const MAGENTA_FG: SetForegroundColor = SetForegroundColor(Color::Magenta);

const NEW_LINE: &str = "\n";
// the extra space is for cursor being 1 in
// when printing menu code
#[cfg(windows)]
const NEW_LINE_RETURN: &str = "\r\n ";
#[cfg(not(windows))]
const NEW_LINE_RETURN: &str = "\n\r ";

pub const BRANCH_HAS_NEXT: &str = "├──";
pub const BRANCH_END: &str = "└──";
pub const VER_LINE_SPACER: &str = "│  ";
pub const SPACER: &str = "   ";

const MATCHED_COLORS: [SetForegroundColor; 3] = [GREEN_FG, MAGENTA_FG, RED_FG];

pub fn get_terminator(is_menu: bool) -> String {
    if is_menu {
        return format!("{}{}{}", RESET, NEW_LINE_RETURN, DEFAULT_FG);
    }
    NEW_LINE.to_string()
}

pub fn get_reset(is_menu: bool) -> String {
    if is_menu {
        format!("{}{}", DEFAULT_FG, NO_BOLD)
    } else {
        format!("{}{}", RESET_COLOR, NO_BOLD)
    }
}

pub fn get_color(i: usize) -> SetForegroundColor {
    MATCHED_COLORS[i % MATCHED_COLORS.len()]
}

pub fn dir_name(name: &str) -> StyledContent<&str> {
    name.with(Color::Blue).attribute(Attribute::Bold)
}

pub fn file_name(name: &str) -> StyledContent<&str> {
    name.with(Color::Cyan).attribute(Attribute::Bold)
}

pub fn error_prefix() -> String {
    format!("{}{}Error: {}", BOLD, RED_FG, RESET)
}
