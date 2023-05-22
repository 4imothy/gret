// SPDX-License-Identifier: GPL-3.0

use crate::formats;
use crate::ignores_parser::parse_for_ignores;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::rc::Rc;

// TODO replace all the Vec::new with options, not every dir has a found file

// Need to search a directory fully, when I find a child directory search that one
// fully as well when a file is found then search it, if that file has the target
// phrase then in the directory it is in add this file to the list of matches
// and trail up the parents till you can add to the highest up directory that is not
// yet in the found tree.

// To do this need to pass a mutable dir to each search of a directory as it needs children added
// to it. Each directory also passed to its file's searches needs to be mutable so we can keep track of
// whether it has been added to the found tree and so its list of matched files can be updated..

// parent
// type DirWeakPointer = Weak<RefCell<Directory>>;
// children
type DirPointer<'a> = Rc<RefCell<Directory<'a>>>;

struct Directory<'a> {
    parent: Option<&'a DirPointer<'a>>,
    // the directories that have a
    children: Vec<DirPointer<'a>>,
    found_files: Vec<File<'a>>,
    name: String,
    to_add: bool,
}

struct File<'a> {
    name: String,
    lines: Vec<&'a str>,
}

// TODO add other denoters that are used, HACK, FIXME
const TODO_BYTES: [u8; 4] = [b'T', b'O', b'D', b'O'];

// since the children directories can also have .gitignores
// need some clever way to keep track of each directories gitignore
// files and directories can be ignored
pub fn search(path: PathBuf, is_dir: bool) -> std::io::Result<()> {
    // to store the number of spaces before a dir, file, line
    // search the path for `TODO` and write it out
    if !is_dir {
        let file_name = get_name_as_string(&path);
        search_file(path, file_name, None);
        return Ok(());
    } else {
        let paths = path.read_dir().unwrap();
        let name = get_name_as_string(&path);
        let top_dir = Directory {
            found_files: Vec::new(),
            children: Vec::new(),
            parent: None,
            to_add: true,
            name,
        };
        let td_ref = Rc::new(RefCell::new(top_dir));
        search_dir(td_ref, paths);
    }

    Ok(())
}

fn search_dir(mut d_ref: DirPointer, paths: std::fs::ReadDir) -> std::io::Result<()> {
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
            // skip if this name is in the gitignore
            // TODO make this work with ignores for files that are deep, a/b/t.txt
            continue;
        }
        match read_dir {
            Ok(read) => {
                // is a directory
                let child_dir = Directory {
                    // parent: Option<Weak<RefCell<Directory<'a>>>>,
                    // children: Vec<Rc<RefCell<Directory<'a>>>>,
                    parent: Some(&d_ref),
                    children: Vec::new(),
                    found_files: Vec::new(),
                    to_add: true,
                    name,
                };
                // dir.children.push(Rc::new(RefCell::new(child_dir)));
                let cd_ref = Rc::new(RefCell::new(child_dir));
                d_ref.borrow().children.push(cd_ref);
                search_dir(cd_ref, read);
            }
            Err(_) => {
                // is a file, print_dir is changed when the dir has been printed once
                search_file(path_buf, name, Some(d_ref));
            }
        }
    }

    Ok(())
}

fn search_file(path: PathBuf, file_name: String, mut directory: Option<DirPointer>) {
    let mut file = File {
        lines: Vec::new(),
        name: file_name,
    };
    let contents: Vec<u8> = fs::read(path).expect("Failed to read file");
    // if a non text file than just return
    if !contents.is_ascii() {
        return;
    }

    let mut line_start = 0;
    for (i, &byte) in contents.iter().enumerate() {
        if byte == b'\n' {
            if line_contains_bytes(&contents[line_start..i]) {
                // add all the parents to the founds until one was added
                let line =
                    std::str::from_utf8(&contents[line_start..i]).expect("Failed to decode line");
                file.lines.push(line.trim());
            }
            // the start of the next line is the char after the \n
            line_start = i + 1;
        }
    }
    // to handle when no newline at end of the file
    if line_start < contents.len() {
        if line_contains_bytes(&contents[line_start..]) {
            let line = std::str::from_utf8(&contents[line_start..]).expect("Failed to decode line");
            file.lines.push(line.trim());
        }
    }

    // if there were any matches
    if file.lines.len() > 0 {
        // can assume founds exists when directory exists
        if let Some(mut d_ref) = directory.take() {
            // d_ref.borrow().children.push(cd_ref);
            d_ref.borrow().found_files.push(file);
            // while has a parent and it is still not in the current found tree
            while d_ref.borrow().parent.is_some() && d_ref.borrow().to_add {
                d_ref.borrow().to_add = false;
                d_ref = *d_ref.borrow().parent.unwrap();
                // let new_current = current.borrow().parent.as_ref().unwrap().uprade().unwrap();
            }
            // push the most parent directory into it
            // then when printing trail down the children
        }
    }
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

fn get_name_as_string(path: &PathBuf) -> String {
    path.file_name()
        .expect("Unable to get file name")
        .to_os_string()
        .into_string()
        .expect("Unable to convert file name to string")
}
