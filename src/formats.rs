// SPDX-License-Identifier: Unlicense

use crossterm::style::{
    Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    StyledContent, Stylize,
};

pub const MENU_SELECTED: SetBackgroundColor = SetBackgroundColor(Color::DarkGrey);
// pub const MENU_SELECTED: SetAttribute = SetAttribute(Attribute::);

pub const RESET: SetAttribute = SetAttribute(Attribute::Reset);
pub const RESET_COLOR: ResetColor = ResetColor;
pub const NO_BOLD: SetAttribute = SetAttribute(Attribute::NormalIntensity);
pub const BOLD: SetAttribute = SetAttribute(Attribute::Bold);
const RED_FG: SetForegroundColor = SetForegroundColor(Color::Red);
const GREEN_FG: SetForegroundColor = SetForegroundColor(Color::Green);
pub const LINE_NUMBER_FG: SetForegroundColor = SetForegroundColor(Color::Yellow);
const MAGENTA_FG: SetForegroundColor = SetForegroundColor(Color::Magenta);

const NEW_LINE: &str = "\n";
// the extra space is for cursor being 1 in
// when printing menu code
const NEW_LINE_RETURN: &str = "\n\r ";

pub const BRANCH_HAS_NEXT: &str = "├──";
pub const BRANCH_END: &str = "└──";
pub const VER_LINE_SPACER: &str = "│  ";
pub const SPACER: &str = "   ";

const MATCHED_COLORS: [SetForegroundColor; 3] = [GREEN_FG, MAGENTA_FG, RED_FG];

pub fn get_terminator(is_menu: bool) -> String {
    if is_menu {
        return NEW_LINE_RETURN.to_string();
    }
    NEW_LINE.to_string()
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
