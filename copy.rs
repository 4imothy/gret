// SPDX-License-Identifier: GPL-3.0

use crate::formats;
use crate::ignores_parser::parse_for_ignores;
use std::collections::HashSet;
use std::fs::{self, DirEntry};
use std::io::{self, StdoutLock, Write};
use std::path::PathBuf;

// Current idea for printing dir, create a list of some dir struct
// each one has whether it should be printed and its name
// when a match is found in a file go up the dirs until one is found
// which was already printed, then print from that point onward

#[derive(Debug)]
struct Directory {
    parent: Option<Box<Directory>>,
    name: String,
    print: bool,
}

// TODO add other denoters that are used, HACK, FIXME
const TODO_BYTES: [u8; 4] = [b'T', b'O', b'D', b'O'];

// since the children directories can also have .gitignores
// need some clever way to keep track of each directories gitignore
// files and directories can be ignored
pub fn search(path: PathBuf, is_dir: bool) -> io::Result<()> {
    let mut handle: StdoutLock = std::io::stdout().lock();
    // to store the number of spaces before a dir, file, line
    let mut depth: usize = 0;
    // search the path for `TODO` and write it out
    if !is_dir {
        let file_name = get_name_as_string(&path);
        search_file(&mut handle, path, file_name, None, &mut depth)?;
    } else {
        let paths = path.read_dir().unwrap();
        let name = get_name_as_string(&path);
        let dir = Directory {
            parent: None,
            print: true,
            name,
        };
        _ = search_dir(&mut handle, dir, paths, &mut depth)?;
    }
    handle.flush()?;

    Ok(())
}

fn search_dir(
    stream: &mut StdoutLock,
    mut dir: Directory,
    paths: std::fs::ReadDir,
    depth: &mut usize,
) -> io::Result<Directory> {
    // if a match is found should you print the dir
    // check if there is a .gitignore or a .ignore file and construct a ignored hashmap if there is
    let entries: Vec<DirEntry> = paths.collect::<Result<Vec<_>, _>>()?;
    let mut ignore_names = HashSet::new();
    parse_for_ignores(&mut ignore_names, &entries);

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
                let new_dir = Directory {
                    parent: Some(Box::new(dir)),
                    print: true,
                    name,
                };
                dir = search_dir(stream, new_dir, read, depth)?;
            }
            Err(_) => {
                // is a file, print_dir is changed when the dir has been printed once
                dir = search_file(stream, path_buf, name, Some(dir), depth)?.unwrap();
            }
        }
    }

    return Ok(dir);
}

fn search_file(
    stream: &mut StdoutLock,
    path: PathBuf,
    file_name: String,
    mut directory: Option<Directory>,
    depth: &mut usize,
) -> io::Result<Option<Directory>> {
    let contents: Vec<u8> = fs::read(path).expect("Failed to read file");
    // if a non text file than just return
    if !contents.is_ascii() {
        return Ok(directory);
    }

    let mut line_start = 0;
    let mut is_first = true;
    for (i, &byte) in contents.iter().enumerate() {
        if byte == b'\n' {
            if line_contains_bytes(&contents[line_start..i]) {
                let mut to_print: Vec<String> = Vec::new();
                if let Some(mut dir) = directory.take() {
                    while dir.print && dir.parent.is_some() {
                        to_print.push(dir.name);
                        dir.print = false;
                        dir = *dir.parent.unwrap();
                    }
                    if dir.print {
                        to_print.push(dir.name);
                    }
                    to_print.reverse();
                    for name in to_print {
                        write_dir_name(stream, &name, depth)?;
                    }
                }

                if is_first {
                    write_file_name(stream, &file_name, depth)?;
                    is_first = false;
                }
                let line =
                    std::str::from_utf8(&contents[line_start..i]).expect("Failed to decode line");
                write_matched_line(stream, line, depth)?;
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
                write_file_name(stream, &file_name, depth)?;
            }
            write_matched_line(stream, line, depth)?;
        }
    }

    Ok(directory)
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

fn write_dir_name(stream: &mut StdoutLock, name: &String, depth: &usize) -> io::Result<()> {
    if *depth == 0 {
        return writeln!(stream, "{}", name);
    }
    for _ in 0..*depth {
        write!(stream, " ")?;
    }
    writeln!(stream, "{}{}", formats::BRANCH_CHAR, name)
}

fn write_file_name(stream: &mut StdoutLock, name: &String, depth: &usize) -> io::Result<()> {
    for _ in 0..*depth {
        write!(stream, " ")?;
    }
    writeln!(stream, "{}{}", formats::BRANCH_CHAR, name)
}

fn write_matched_line(stream: &mut StdoutLock, line: &str, depth: &usize) -> io::Result<()> {
    for _ in 0..*depth {
        write!(stream, " ")?;
    }
    writeln!(stream, "{}{}", formats::BRANCH_CHAR, line.trim_start())
}

fn get_name_as_string(path: &PathBuf) -> String {
    path.file_name()
        .expect("Unable to get file name")
        .to_os_string()
        .into_string()
        .expect("Unable to convert file name to string")
}

