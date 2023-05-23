use crate::formats::{BRANCH_END, BRANCH_HAS_NEXT, SPACER, VER_LINE_SPACER};
use crate::searcher::DirPointer;
use crate::searcher::File;
use std::io::{self, Write};

pub fn print_directory(
    out: &mut io::StdoutLock,
    dir_ptr: DirPointer,
    depth: usize,
    mut prefix: String,
    parent_has_next: bool,
) -> io::Result<()> {
    let dir = dir_ptr.borrow();

    let files = &dir.found_files;
    let children = dir.children.clone();
    if depth != 0 {
        if parent_has_next {
            writeln!(out, "{}{}{}", prefix, BRANCH_HAS_NEXT, dir.name)?;
            prefix += VER_LINE_SPACER;
        } else {
            writeln!(out, "{}{}{}", prefix, BRANCH_END, dir.name)?;
            prefix += SPACER;
        }
    } else {
        writeln!(out, "{}{}", prefix, dir.name)?;
    }

    let mut i: usize = 0;
    let clen = children.len();
    let flen = files.len();
    for child in children {
        i += 1;
        if i != clen || flen > 0 {
            print_directory(out, child, depth + 1, prefix.clone(), true)?;
        } else {
            print_directory(out, child, depth + 1, prefix.clone(), false)?;
        }
    }
    i = 0;
    for file in files {
        i += 1;
        if i != flen {
            print_file(out, file, depth + 1, prefix.clone(), true)?;
        } else {
            print_file(out, file, depth + 1, prefix.clone(), false)?;
        }
    }

    Ok(())
}

pub fn print_file(
    out: &mut io::StdoutLock,
    file: &File,
    depth: usize,
    mut prefix: String,
    parent_has_next: bool,
) -> io::Result<()> {
    if parent_has_next {
        writeln!(out, "{}{}{}", prefix, BRANCH_HAS_NEXT, file.name)?;
        prefix += VER_LINE_SPACER;
    } else {
        writeln!(out, "{}{}{}", prefix, BRANCH_END, file.name)?;
        prefix += SPACER;
    }

    let len = file.lines.len();
    let mut i = 0;
    for line in &file.lines {
        // for _ in 0..depth {
        //     write!(out, "{}", SPACER)?;
        // }
        i += 1;
        if i != len {
            write!(out, "{}{}", prefix, BRANCH_HAS_NEXT)?;
        } else {
            write!(out, "{}{}", prefix, BRANCH_END)?;
        }
        println!("line: {}", line);
    }

    Ok(())
}
