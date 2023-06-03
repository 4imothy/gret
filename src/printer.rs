// SPDX-License-Identifier: Unlicense

use crate::formats::{BOLD, DIR_COLOR, FILE_COLOR, RESET as STYLE_RESET};
use crate::formats::{BRANCH_END, BRANCH_HAS_NEXT, SPACER, VER_LINE_SPACER};
use crate::searcher::DirPointer;
use crate::searcher::Directory;
use crate::searcher::File;
use crate::Errors;
use crate::CONFIG;
use std::io::{self, Write};

pub fn start_print_directory(out: &mut io::StdoutLock, dir_ptr: DirPointer) -> Result<(), Errors> {
    let prefix = "".to_string();
    let dir = dir_ptr.borrow();
    write_dir_name(out, &dir)?;

    handle_descendants(out, dir, prefix)?;

    Ok(())
}

fn handle_descendants(
    out: &mut io::StdoutLock,
    dir: std::cell::Ref<'_, Directory>,
    prefix: String,
) -> Result<(), Errors> {
    let files = &dir.found_files;
    let children = dir.children.clone();
    let mut i: usize = 0;
    let clen = children.len();
    let flen = files.len();
    for child in children {
        i += 1;
        // check if it has a next file
        if i != clen || flen > 0 {
            print_directory(out, child, prefix.clone(), true)?;
        } else {
            print_directory(out, child, prefix.clone(), false)?;
        }
    }
    i = 0;
    for file in files {
        i += 1;
        // check if it has a next file
        if i != flen {
            print_file(out, file, prefix.clone(), true)?;
        } else {
            print_file(out, file, prefix.clone(), false)?;
        }
    }
    Ok(())
}

fn print_directory(
    out: &mut io::StdoutLock,
    dir_ptr: DirPointer,
    mut prefix: String,
    parent_has_next: bool,
) -> Result<(), Errors> {
    let dir = dir_ptr.borrow();

    if parent_has_next {
        write!(out, "{}{}", prefix, BRANCH_HAS_NEXT).map_err(|_| Errors::CantWrite)?;
        write_dir_name(out, &dir)?;
        prefix += VER_LINE_SPACER;
    } else {
        write!(out, "{}{}", prefix, BRANCH_END).map_err(|_| Errors::CantWrite)?;
        write_dir_name(out, &dir)?;
        prefix += SPACER;
    }

    handle_descendants(out, dir, prefix)?;

    Ok(())
}

fn print_file(
    out: &mut io::StdoutLock,
    file: &File,
    mut prefix: String,
    parent_has_next: bool,
) -> Result<(), Errors> {
    if parent_has_next {
        write!(out, "{}{}", prefix, BRANCH_HAS_NEXT).map_err(|_| Errors::CantWrite)?;
        write_file_name(out, &file)?;
        prefix += VER_LINE_SPACER;
    } else {
        write!(out, "{}{}", prefix, BRANCH_END).map_err(|_| Errors::CantWrite)?;
        write_file_name(out, &file)?;
        prefix += SPACER;
    }

    let len = file.lines.len();
    let mut i = 0;
    for line in &file.lines {
        i += 1;
        if i != len {
            writeln!(out, "{}{}{}", prefix, BRANCH_HAS_NEXT, line)
                .map_err(|_| Errors::CantWrite)?;
        } else {
            writeln!(out, "{}{}{}", prefix, BRANCH_END, line).map_err(|_| Errors::CantWrite)?;
        }
    }

    Ok(())
}

pub fn print_single_file(out: &mut io::StdoutLock, file: &File) -> Result<(), Errors> {
    write_file_name(out, &file)?;

    let len = file.lines.len();
    let mut i = 0;
    for line in &file.lines {
        i += 1;
        if i != len {
            writeln!(out, "{}{}", BRANCH_HAS_NEXT, line).map_err(|_| Errors::CantWrite)?;
        } else {
            writeln!(out, "{}{}", BRANCH_END, line).map_err(|_| Errors::CantWrite)?;
        }
    }
    Ok(())
}

fn write_file_name(out: &mut io::StdoutLock, file: &File) -> Result<(), Errors> {
    if CONFIG.styled {
        write!(out, "{FILE_COLOR}{BOLD}").map_err(|_| Errors::CantWrite)?;
    }

    if let Some(linked) = &file.linked {
        if CONFIG.styled {
            write!(out, "{}{STYLE_RESET} -> ", file.name).map_err(|_| Errors::CantWrite)?;
        } else {
            write!(out, "{} -> ", file.name).map_err(|_| Errors::CantWrite)?
        }
        if CONFIG.styled {
            write!(
                out,
                "{FILE_COLOR}{BOLD}{}{STYLE_RESET}",
                linked.to_string_lossy()
            )
            .map_err(|_| Errors::CantWrite)?;
        } else {
            write!(out, "{}", linked.to_string_lossy()).map_err(|_| Errors::CantWrite)?;
        }
    } else {
        if CONFIG.styled {
            write!(out, "{}{STYLE_RESET}", file.name).map_err(|_| Errors::CantWrite)?;
        } else {
            write!(out, "{}", file.name).map_err(|_| Errors::CantWrite)?;
        }
    }
    if CONFIG.show_count {
        write!(out, ": {}", file.lines.len()).map_err(|_| Errors::CantWrite)?;
    }
    writeln!(out).map_err(|_| Errors::CantWrite)?;

    Ok(())
}

fn write_dir_name(out: &mut io::StdoutLock, dir: &Directory) -> Result<(), Errors> {
    if CONFIG.styled {
        write!(out, "{}{}{}{}", DIR_COLOR, BOLD, dir.name, STYLE_RESET)
            .map_err(|_| Errors::CantWrite)?;
    } else {
        write!(out, "{}", dir.name).map_err(|_| Errors::CantWrite)?;
    }
    if CONFIG.show_count {
        write!(out, ": {}", dir.found_files.len() + dir.children.len())
            .map_err(|_| Errors::CantWrite)?;
    }
    writeln!(out).map_err(|_| Errors::CantWrite)?;
    Ok(())
}
