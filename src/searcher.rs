// SPDX-License-Identifier: GPL-3.0

use crate::ignores_parser::parse_for_ignores;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::rc::{Rc, Weak};

/*
To read for the TODOs, we read each entry of a directory. If the entry
is a directory, we seacrh it. After done with the children we come back
and finish the search on this directory. When a file is encountered
with the phrase TODO we add the file to the directories list of files.
Then we chain up the parents of this directory to add them to the children
of the earliest directory that was already added as a child to it's parent.
*/

pub type DirPointer = Rc<RefCell<Directory>>;
pub type WeakDirPointer = Weak<RefCell<Directory>>;

pub struct Directory {
    pub parent: Option<WeakDirPointer>,
    // the directories that have a matched file
    pub children: Vec<DirPointer>,
    pub found_files: Vec<File>,
    pub name: String,
    pub to_add: bool,
}

pub struct File {
    pub name: String,
    pub lines: Vec<String>,
}

// TODO add other denoters that are used, HACK, FIXME, TODO
const TODO_BYTES: [u8; 4] = [b'T', b'O', b'D', b'O'];

// since the children directories can also have .gitignores
// need some clever way to keep track of each directories gitignore
// files and directories can be ignored

pub fn search_singe_file(path: PathBuf) -> Option<File> {
    let name = get_name_as_string(&path);
    let mut file = File {
        // this does not allocate memory
        lines: Vec::new(),
        name,
    };
    let contents: Vec<u8> = fs::read(path).expect("Failed to read file");

    // if a non text file than just return
    // TODO make this call more effecient
    if !is_text(&contents) {
        return None;
    }

    add_matches(&mut file, contents);

    Some(file)
}

fn is_text(contents: &Vec<u8>) -> bool {
    match std::str::from_utf8(contents) {
        Ok(_) => return true,
        Err(_) => return false,
    };
}

pub fn start_search_dir(path: PathBuf) -> std::io::Result<DirPointer> {
    // to store the number of spaces before a dir, file, line
    // search the path for `TODO` and write it out
    let paths = path.read_dir().unwrap();
    let name = get_name_as_string(&path);
    let top_dir = Directory {
        // this doesn't allocate memory
        found_files: Vec::new(),
        children: Vec::new(),
        parent: None,
        to_add: true,
        name,
    };
    let td_ref = Rc::new(RefCell::new(top_dir));
    search_dir(td_ref.clone(), paths)?;

    Ok(td_ref)
}

fn search_dir(d_ref: DirPointer, paths: std::fs::ReadDir) -> std::io::Result<()> {
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
            continue;
        }
        match read_dir {
            Ok(read) => {
                // is a directory
                let child_dir = Directory {
                    // parent: Option<Weak<RefCell<Directory<'a>>>>,
                    // children: Vec<Rc<RefCell<Directory<'a>>>>,
                    parent: Some(Rc::downgrade(&d_ref)),
                    children: Vec::new(),
                    found_files: Vec::new(),
                    to_add: true,
                    name,
                };
                let cd_ref = Rc::new(RefCell::new(child_dir));
                // d_ref.borrow_mut().children.push(cd_ref.clone());
                search_dir(cd_ref, read)?;
            }
            Err(_) => {
                // is a file, print_dir is changed when the dir has been printed once
                search_file(path_buf, name, Some(d_ref.clone()));
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
    // TODO make this call more effecient
    if !is_text(&contents) {
        return;
    }

    add_matches(&mut file, contents);

    // if there were any matches
    if file.lines.len() > 0 {
        // can assume founds exists when directory exists
        if let Some(mut d_ref) = directory.take() {
            d_ref.borrow_mut().found_files.push(file);
            // while has a parent and it is still not in the current found tree
            while d_ref.borrow().parent.is_some() && d_ref.borrow().to_add {
                d_ref.borrow_mut().to_add = false;
                let new_d_ref = d_ref.borrow().parent.clone().unwrap().upgrade().unwrap();
                new_d_ref.borrow_mut().children.push(d_ref);
                d_ref = new_d_ref;
                // let new_current = current.borrow().parent.as_ref().unwrap().uprade().unwrap();
            }
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

fn add_matches(file: &mut File, contents: Vec<u8>) {
    let mut line_start = 0;
    for (i, &byte) in contents.iter().enumerate() {
        if byte == b'\n' {
            if line_contains_bytes(&contents[line_start..i]) {
                // add all the parents to the founds until one was added
                let line =
                    std::str::from_utf8(&contents[line_start..i]).expect("Failed to decode line");
                file.lines.push(line.trim().to_string());
            }
            // the start of the next line is the char after the \n
            line_start = i + 1;
        }
    }
    // to handle when no newline at end of the file
    if line_start < contents.len() {
        if line_contains_bytes(&contents[line_start..]) {
            let line = std::str::from_utf8(&contents[line_start..]).expect("Failed to decode line");
            file.lines.push(line.trim().to_string());
        }
    }
}
