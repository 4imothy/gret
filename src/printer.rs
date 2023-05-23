use crate::formats::{BRANCH_END, BRANCH_HAS_NEXT, SPACER, VER_LINE_SPACER};
use crate::searcher::DirPointer;
use crate::searcher::Directory;
use crate::searcher::File;
use crate::Errors;
use std::io::{self, Write};

pub fn start_print_directory(out: &mut io::StdoutLock, dir_ptr: DirPointer) -> Result<(), Errors> {
    let prefix = "".to_string();
    let dir = dir_ptr.borrow();
    writeln!(out, "{}", dir.name).map_err(|_| Errors::CantWrite)?;

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
        writeln!(out, "{}{}{}", prefix, BRANCH_HAS_NEXT, dir.name)
            .map_err(|_| Errors::CantWrite)?;
        prefix += VER_LINE_SPACER;
    } else {
        writeln!(out, "{}{}{}", prefix, BRANCH_END, dir.name).map_err(|_| Errors::CantWrite)?;
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
        writeln!(out, "{}{}{}", prefix, BRANCH_HAS_NEXT, file.name)
            .map_err(|_| Errors::CantWrite)?;
        prefix += VER_LINE_SPACER;
    } else {
        writeln!(out, "{}{}{}", prefix, BRANCH_END, file.name).map_err(|_| Errors::CantWrite)?;
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
    writeln!(out, "{}", file.name).map_err(|_| Errors::CantWrite)?;

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
