// SPDX-License-Identifier: CC-BY-4.0

use crate::formats::{self, BRANCH_END, BRANCH_HAS_NEXT, SPACER, VER_LINE_SPACER};
use crate::searcher::{Directory, File, MatchedLine, Searched};
use crate::CONFIG;
use std::io::{self, Write};

fn write_file_path(out: &mut impl Write, file: &File) -> io::Result<()> {
    let path: &str = &file.path.to_string_lossy();
    if let Some(linked) = &file.linked {
        if CONFIG.styled {
            write!(out, "{} -> ", formats::file_name(path))?;
        } else {
            write!(out, "{} -> ", path)?
        }
        if CONFIG.styled {
            write!(out, "{}", formats::file_name(&linked.to_string_lossy()))?;
        } else {
            write!(out, "{}", linked.to_string_lossy())?;
        }
    } else {
        if CONFIG.styled {
            write!(out, "{}", formats::file_name(path))?;
        } else {
            write!(out, "{}", path)?;
        }
    }
    if CONFIG.show_count {
        write!(out, ": {}", file.lines.len())?;
    }
    new_line(out)?;

    Ok(())
}

pub fn write_results(out: &mut impl Write, result: &Searched) -> io::Result<()> {
    let prefix = "".into();
    match &result {
        Searched::Dir(dirs) => {
            write_dir(out, dirs.get(0).unwrap(), prefix, dirs)?;
        }
        Searched::File(file) => {
            write_file(out, &file, prefix, false)?;
        }
    }

    Ok(())
}

fn write_dir(
    out: &mut impl Write,
    dir: &Directory,
    prefix: String,
    dirs: &Vec<Directory>,
) -> io::Result<()> {
    let children = &dir.children;
    let files = &dir.files;
    let flen = files.len();
    let clen = children.len();
    if (clen > 0 || flen > 0) && !CONFIG.just_files {
        write_dir_name(out, &dir)?;
    }
    let mut i: usize = 0;
    for child_id in children {
        i += 1;
        // check if it has a next file
        let dir = dirs.get(*child_id).unwrap();
        if i != clen || flen > 0 {
            if !CONFIG.just_files {
                write!(out, "{}{}", prefix, BRANCH_HAS_NEXT)?;
            }
            write_dir(out, dir, (prefix.clone() + VER_LINE_SPACER).clone(), dirs)?;
        } else {
            if !CONFIG.just_files {
                write!(out, "{}{}", prefix, BRANCH_END)?;
            }
            write_dir(out, dir, (prefix.clone() + SPACER).clone(), dirs)?;
        }
    }
    i = 0;
    for file in files {
        i += 1;
        // check if it has a next file
        if i != flen {
            write_file(out, file, prefix.clone(), true)?;
        } else {
            write_file(out, file, prefix.clone(), false)?;
        }
    }
    Ok(())
}

fn write_file(
    out: &mut impl Write,
    file: &File,
    mut prefix: String,
    parent_has_next: bool,
) -> io::Result<()> {
    if CONFIG.just_files {
        return write_file_path(out, file);
    }
    let len = file.lines.len();
    if prefix == "" {
        write_file_name(out, &file)?;
    } else if parent_has_next {
        write!(out, "{}{}", prefix, BRANCH_HAS_NEXT)?;
        write_file_name(out, &file)?;
        prefix += VER_LINE_SPACER;
    } else {
        write!(out, "{}{}", prefix, BRANCH_END)?;
        write_file_name(out, &file)?;
        prefix += SPACER;
    }

    let mut i = 0;
    for line_match in file.lines.iter() {
        i += 1;
        if i != len {
            write!(out, "{}{}", prefix, BRANCH_HAS_NEXT,)?;
        } else {
            write!(out, "{}{}", prefix, BRANCH_END)?;
        }
        print_line(out, line_match)?;
        new_line(out)?;
    }

    Ok(())
}

pub fn write_file_name(out: &mut impl Write, file: &File) -> io::Result<()> {
    if let Some(linked) = &file.linked {
        if CONFIG.styled {
            write!(out, "{} -> ", formats::file_name(&file.name))?;
        } else {
            write!(out, "{} -> ", file.name)?
        }
        if CONFIG.styled {
            write!(out, "{}", formats::file_name(&linked.to_string_lossy()))?;
        } else {
            write!(out, "{}", linked.to_string_lossy())?;
        }
    } else {
        if CONFIG.styled {
            write!(out, "{}", formats::file_name(&file.name))?;
        } else {
            write!(out, "{}", file.name)?;
        }
    }
    if CONFIG.show_count {
        write!(out, ": {}", file.lines.len())?;
    }
    new_line(out)?;

    Ok(())
}

fn write_dir_name(out: &mut impl Write, dir: &Directory) -> io::Result<()> {
    if CONFIG.styled {
        write!(out, "{}", formats::dir_name(&dir.name))?;
    } else {
        write!(out, "{}", dir.name)?;
    }
    if CONFIG.show_count {
        write!(out, ": {}", dir.files.len() + dir.children.len())?;
    }
    new_line(out)?;
    Ok(())
}

pub fn print_line(out: &mut impl Write, line_match: &MatchedLine) -> std::io::Result<()> {
    let line: &[u8] = &line_match.contents;
    // let line: &[u8] = &line_match.contents;
    let line_num = line_match.line_num;
    if !CONFIG.styled {
        if CONFIG.show_line_number {
            write!(out, "{}: ", line_num)?;
        }
        write!(out, "{}", String::from_utf8_lossy(&line).trim())?;
        return Ok(());
    }
    let mut last_match = 0;
    if CONFIG.show_line_number {
        if CONFIG.styled {
            write!(out, "{}{}", formats::LINE_NUMBER_FG, formats::BOLD)?;
        }
        write!(out, "{}: ", line_num)?;
        if CONFIG.styled {
            write_resets(out)?;
        }
    }
    for m in line_match.matches.iter() {
        write!(
            out,
            "{}",
            String::from_utf8_lossy(&line[last_match..m.start])
        )?;
        last_match = m.end;
        if CONFIG.styled {
            write!(out, "{}{}", formats::get_color(m.regex_id), formats::BOLD,)?;
        }
        write!(out, "{}", String::from_utf8_lossy(&line[m.start..m.end]))?;
        if CONFIG.styled {
            write_resets(out)?;
        }
    }
    write!(out, "{}", String::from_utf8_lossy(&line[last_match..]))?;

    Ok(())
}

fn write_resets(out: &mut impl Write) -> io::Result<()> {
    write!(out, "{}", CONFIG.reset)
}

fn new_line(out: &mut impl Write) -> io::Result<()> {
    write!(out, "{}", CONFIG.terminator)
}
