// SPDX-License-Identifier: GPL-3.0

use crate::formats;
use crate::ignores_parser::parse_for_ignores;
use std::collections::HashSet;
use std::fs::{self, DirEntry};
use std::io::{self, StdoutLock, Write};
use std::path::PathBuf;

// TODO add other denoters that are used, HACK,
const TODO_BYTES: [u8; 4] = [b'T', b'O', b'D', b'O'];

// since the children directories can also have .gitignores
// need some clever way to keep track of each directories gitignore
// files and directories can be ignored
pub fn search(path: PathBuf, is_dir: bool) -> io::Result<()> {
    let mut handle: StdoutLock = std::io::stdout().lock();
    // search the path for `TODO` and write it out
    if !is_dir {
        let file_name = get_name_as_string(&path);
        search_file(&mut handle, path, file_name, None, &mut false)?;
    } else {
        let paths = path.read_dir().unwrap();
        search_dir(&mut handle, get_name_as_string(&path), paths)?;
    }
    handle.flush()?;

    Ok(())
}

fn search_dir(
    stream: &mut StdoutLock,
    dir_name: String,
    paths: std::fs::ReadDir,
) -> io::Result<()> {
    // if a match is found should you print the dir
    // check if there is a .gitignore or a .ignore file and construct a ignored hashmap if there is
    let entries: Vec<DirEntry> = paths.collect::<Result<Vec<_>, _>>()?;
    let mut ignore_names = HashSet::new();
    parse_for_ignores(&mut ignore_names, &entries);

    let mut print_dir = true;
    for entry in entries {
        let path_buf = entry.path();
        let read_dir = path_buf.read_dir();
        let name: String = get_name_as_string(&path_buf);
        if ignore_names.contains(&name) {
            continue;
        }
        match read_dir {
            Ok(read) => {
                // is a directory
                search_dir(stream, name, read)?;
            }
            Err(_) => {
                // is a file, print_dir is changed when the dir has been printed once
                search_file(stream, path_buf, name, Some(&dir_name), &mut print_dir)?;
            }
        }
    }

    Ok(())
}

fn search_file(
    stream: &mut StdoutLock,
    path: PathBuf,
    file_name: String,
    dir_name: Option<&String>,
    print_dir: &mut bool,
) -> io::Result<()> {
    let contents: Vec<u8> = fs::read(path).expect("Failed to read file");
    // if a non text file than just return
    if !contents.is_ascii() {
        return Ok(());
    }

    let mut line_start = 0;
    let mut is_first = true;
    for (i, &byte) in contents.iter().enumerate() {
        if byte == b'\n' {
            if line_contains_bytes(&contents[line_start..i]) {
                if *print_dir {
                    write_dir_name(stream, dir_name.unwrap())?;
                    *print_dir = false;
                }
                if is_first {
                    write_file_name(stream, &file_name)?;
                    is_first = false;
                }
                let line =
                    std::str::from_utf8(&contents[line_start..i]).expect("Failed to decode line");
                write_matched_line(stream, line)?;
            }
            // the start of the next line is the char after the \n
            line_start = i + 1;
        }
    }

    // to handle when no newline at end of the file
    if line_start < contents.len() {
        if line_contains_bytes(&contents[line_start..]) {
            let line = std::str::from_utf8(&contents[line_start..]).expect("Failed to decode line");
            if is_first {
                write_file_name(stream, &file_name)?;
            }
            write_matched_line(stream, line)?;
        }
    }

    Ok(())
}

fn line_contains_bytes(line: &[u8]) -> bool {
    let line_len = line.len();
    let tar_len = TODO_BYTES.len();
    if line_len < tar_len {
        return false;
    }

    for i in 0..=line_len - tar_len {
        if line[i..i + tar_len] == TODO_BYTES {
            return true;
        }
    }

    false
}

fn write_dir_name(stream: &mut StdoutLock, name: &String) -> io::Result<()> {
    write!(stream, "Dir: {}\n\n", name)
}

fn write_file_name(stream: &mut StdoutLock, name: &String) -> io::Result<()> {
    write!(stream, "{}\n", name)
}

fn write_matched_line(stream: &mut StdoutLock, line: &str) -> io::Result<()> {
    // TODO remove all the blank characters at the beginning
    write!(stream, "{}\n", line)
}

fn get_name_as_string(path: &PathBuf) -> String {
    path.file_name()
        .expect("Unable to get file name")
        .to_os_string()
        .into_string()
        .expect("Unable to convert file name to string")
}
