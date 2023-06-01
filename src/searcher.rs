// SPDX-License-Identifier: Unlicense

use crate::Errors;
use formats::{
    BOLD as BOLD_STR, FIXME_COLOR, HACK_COLOR, NOTE_COLOR, RESET as STYLE_RESET, TODO_COLOR,
};
use ignore::WalkBuilder;
use lazy_static::lazy_static;
use memchr::memchr;
use regex::bytes::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub type DirPointer = Rc<RefCell<Directory>>;

struct Pattern {
    text: Vec<u8>,
    regex: Regex,
    color: Vec<u8>,
}

lazy_static! {
    static ref BOLD: Vec<u8> = BOLD_STR.as_bytes().to_vec();
    static ref RESET: Vec<u8> = STYLE_RESET.as_bytes().to_vec();

    static ref PATTERNS: Vec<Pattern> = vec![
        Pattern {
            text: b"TODO".to_vec(),
            regex: Regex::new(r"TODO").unwrap(),
            color: TODO_COLOR.as_bytes().to_vec(),
        },// TODO
        Pattern {
            text: b"NOTE".to_vec(),
            regex: Regex::new(r"NOTE").unwrap(),
            color: NOTE_COLOR.as_bytes().to_vec(),
        },// NOTE
        Pattern {
            text: b"HACK".to_vec(),
            regex: Regex::new(r"HACK").unwrap(),
            color: HACK_COLOR.as_bytes().to_vec(),
        },// HACK
        Pattern {
            text: b"FIXME".to_vec(),
            regex: Regex::new(r"FIXME").unwrap(),
            color: FIXME_COLOR.as_bytes().to_vec(),
        },// FIXME
    ];
}

pub struct Directory {
    // the directories that have a matched file
    pub children: Vec<DirPointer>,
    pub found_files: Vec<File>,
    pub name: String,
    pub to_add: bool,
}

impl Directory {
    fn new(name: String) -> Directory {
        Directory {
            found_files: Vec::new(),
            children: Vec::new(),
            to_add: true,
            name,
        }
    }
}

pub struct File {
    pub name: String,
    pub lines: Vec<String>,
    // if it is a symbolic link then stored the
    // path that is linked to
    pub linked: Option<PathBuf>,
}

impl File {
    fn add_matches(&mut self, contents: Vec<u8>) {
        // check if it is a binary file
        if memchr(0, &contents).is_some() {
            return;
        }
        let lines = contents.split(|&byte| byte == b'\n'); // Split contents into lines

        for line in lines {
            let mut temp_copy: Vec<u8> = line.clone().to_vec();
            let mut was_match = false;
            for pattern in PATTERNS.iter() {
                let len = temp_copy.len()
                    + BOLD.len()
                    + pattern.color.len()
                    + pattern.text.len()
                    + RESET.len();
                let mut new: Vec<u8> = Vec::with_capacity(len);
                let mut rep: Vec<u8> = Vec::with_capacity(len);
                rep.extend_from_slice(&pattern.color);
                rep.extend_from_slice(&BOLD);
                rep.extend_from_slice(&pattern.text);
                rep.extend_from_slice(&RESET);
                let mut it = pattern.regex.find_iter(&temp_copy).peekable();
                if it.peek().is_none() {
                    continue;
                }
                was_match = true;
                let mut last_match = 0;
                for m in it {
                    new.extend_from_slice(&temp_copy[last_match..m.start()]);
                    new.extend_from_slice(&rep);
                    last_match = m.end();
                }
                new.extend_from_slice(&temp_copy[last_match..]);
                temp_copy = new;
            }
            if was_match {
                self.lines
                    .push(String::from_utf8_lossy(&temp_copy).trim().to_string());
            }
        }
    }
}

pub fn begin_search_on_directory(root_path: PathBuf) -> Result<DirPointer, Errors> {
    // TODO make this an option
    let w = WalkBuilder::new(&root_path).hidden(true).build();
    // this stores every directory whether or not it has a matched file
    let mut directories: HashMap<PathBuf, DirPointer> = HashMap::new();
    let name = get_name_as_string(&root_path).unwrap_or_else(|_| "/".to_string());
    let top_dir = Directory::new(name);
    let td_ref: DirPointer = Rc::new(RefCell::new(top_dir));
    // skip the top directory
    for result in w.skip(1) {
        // Each item yielded by the iterator is either a directory entry or an
        // error, so either print the path or the error.
        match result {
            Ok(entry) => {
                let pb: PathBuf = entry.into_path();
                if pb.is_dir() {
                    let name = get_name_as_string(&pb)?;
                    if directories.get(&pb).is_none() {
                        let new_dir = Directory::new(name);
                        directories.insert(pb, Rc::new(RefCell::new(new_dir)));
                    }
                } else if pb.is_file() {
                    // this returns none if file isn't text or has no matched lines
                    let m_file = search_file(&pb)?;
                    // if the file had matches
                    if let Some(file) = m_file {
                        let m_dir_path = pb.parent();
                        if m_dir_path == Some(&root_path) {
                            // if the parent is none we are in the top directory so add it to that
                            td_ref.borrow_mut().found_files.push(file);
                        } else if let Some(dir_path) = m_dir_path {
                            // while file.parent isnt the root path
                            // we add them to a list to be
                            let mut dir_ref: &DirPointer = directories.get(dir_path).unwrap();

                            dir_ref.borrow_mut().found_files.push(file);
                            let mut m_dir_parent_path = dir_path.parent();
                            // and the to add check
                            while let Some(dir_parent_path) = m_dir_parent_path {
                                if dir_parent_path == root_path {
                                    break;
                                }
                                let parent_ref = directories.get(dir_parent_path).unwrap();
                                if !dir_ref.borrow().to_add {
                                    break;
                                }
                                parent_ref.borrow_mut().children.push(dir_ref.clone());
                                dir_ref.borrow_mut().to_add = false;
                                m_dir_parent_path = dir_parent_path.parent();
                                dir_ref = parent_ref;
                            }
                            if dir_ref.borrow_mut().to_add {
                                td_ref.borrow_mut().children.push(dir_ref.clone());
                                dir_ref.borrow_mut().to_add = false;
                            }
                        }
                    }
                }
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
    Ok(td_ref)
}

pub fn search_file(pb: &PathBuf) -> Result<Option<File>, Errors> {
    // TODO make this an error
    let content_bytes: Vec<u8> = fs::read(&pb).expect("Failed to read file");

    let linked = fs::read_link(&pb).ok().and_then(|target_path| {
        match std::env::var("HOME").ok() {
            Some(home) => {
                // if HOME was found
                target_path
                    .strip_prefix(home)
                    .ok()
                    .map(|clean_path| PathBuf::from("~").join(clean_path))
            }
            None => Some(target_path),
        }
    });

    let mut file = File {
        lines: Vec::new(),
        name: get_name_as_string(&pb)?,
        linked,
    };

    file.add_matches(content_bytes);
    if file.lines.len() == 0 {
        return Ok(None);
    }

    return Ok(Some(file));
}

// TODO make this an OsStr, use smth like
// self.path.file_name().unwrap_or_else(|| self.path.as_os_str())
fn get_name_as_string(path: &PathBuf) -> Result<String, Errors> {
    let name = path.file_name().ok_or(Errors::CantGetName {
        cause: path.clone(),
    })?;

    let stringed_name = name
        .to_os_string()
        .into_string()
        .map_err(|_| Errors::CantGetName {
            cause: path.clone(),
        })?;

    Ok(stringed_name)
}
