// SPDX-License-Identifier: GPL-3.0

// use crate::formats;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

// TODO add other denoters that are used, HACK,
const TODO_BYTES: [u8; 4] = [b'T', b'O', b'D', b'O'];

// since the children directories can also have .gitignores
// need some clever way to keep track of each directories gitignore
// files and directories can be ignored
pub fn search(path: PathBuf, is_dir: bool) -> io::Result<()> {
    let mut stream: BufWriter<io::Stdout> = BufWriter::new(io::stdout());
    // search the path for `TODO` and write it out
    if !is_dir {
        search_file(&mut stream, path, None, &mut false)?;
    } else {
        let paths = path.read_dir().unwrap();
        search_dir(&mut stream, get_name_as_string(&path), paths)?;
    }
    stream.flush().unwrap();

    Ok(())
}

fn search_dir(
    stream: &mut BufWriter<io::Stdout>,
    dir_name: String,
    paths: std::fs::ReadDir,
) -> io::Result<()> {
    // if a match is found should you print the dir
    // check if there is a .gitignore or a .ignore file and construct a ignored hashmap if there is
    let entries: Vec<_> = paths.collect::<Result<Vec<_>, _>>()?;

    for entry in &entries {
        let name = entry.file_name();
        if name == ".gitignore" || name == ".ignore" {
            println!("found");
        }
    }
    let mut print_dir = true;
    for entry in entries {
        let path_buf = entry.path();
        let read_dir = path_buf.read_dir();
        match read_dir {
            Ok(read) => {
                // is a directory
                search_dir(stream, get_name_as_string(&path_buf), read)?;
            }
            Err(_) => {
                // is a file, print_dir is changed when the dir has been printed once
                search_file(stream, path_buf, Some(&dir_name), &mut print_dir)?;
            }
        }
    }

    Ok(())
}

fn search_file(
    stream: &mut BufWriter<io::Stdout>,
    path: PathBuf,
    dir_name: Option<&String>,
    print_dir: &mut bool,
) -> io::Result<()> {
    let file_name = get_name_as_string(&path);
    let contents: Vec<u8> = fs::read(path).expect("Failed to read file");
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

fn write_dir_name(stream: &mut BufWriter<io::Stdout>, name: &String) -> io::Result<()> {
    write!(stream, "Dir: {}\n\n", name)
}

fn write_file_name(stream: &mut BufWriter<io::Stdout>, name: &String) -> io::Result<()> {
    write!(stream, "{}\n", name)
}

fn write_matched_line(stream: &mut BufWriter<io::Stdout>, line: &str) -> io::Result<()> {
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
