// SPDX-License-Identifier: CC-BY-4.0

use crate::Errors;
use crate::CONFIG;
use ignore::WalkBuilder;
use memchr::memchr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub type DirPointer = Rc<RefCell<Directory>>;

// Each directory needs to know its children directories and its matched files
//

pub struct Directory {
    // the directories that have a matched file
    pub children: Vec<DirPointer>,
    pub found_files: Vec<File>,
    pub name: String,
    pub to_add: bool,
    pub path: PathBuf,
}

impl Directory {
    fn new(name: String, path: PathBuf) -> Directory {
        Directory {
            found_files: Vec::new(),
            children: Vec::new(),
            to_add: true,
            path,
            name,
        }
    }
}

pub struct File {
    pub name: String,
    pub lines: Vec<LineMatch>,
    pub linked: Option<PathBuf>,
    pub path: PathBuf,
}

pub enum SearchedTypes {
    Dir(DirPointer),
    File(File),
}

pub struct Match {
    pub regex_id: usize,
    pub start: usize,
    pub end: usize,
}

pub struct LineMatch {
    pub line_num: usize,
    pub contents: Vec<u8>,
    pub matches: Vec<Match>,
}

// Source: https://stackoverflow.com/questions/31101915/how-to-implement-trim-for-vecu8
trait SliceExt {
    fn trim(&self) -> &Self;
}

impl SliceExt for [u8] {
    fn trim(&self) -> &[u8] {
        fn is_whitespace(c: &u8) -> bool {
            *c == b'\t' || *c == b' '
        }

        fn is_not_whitespace(c: &u8) -> bool {
            !is_whitespace(c)
        }

        if let Some(first) = self.iter().position(is_not_whitespace) {
            if let Some(last) = self.iter().rposition(is_not_whitespace) {
                &self[first..last + 1]
            } else {
                unreachable!();
            }
        } else {
            &[]
        }
    }
}

impl File {
    fn add_matches(&mut self, contents: Vec<u8>) {
        // check if it is a binary file
        if memchr(0, &contents).is_some() {
            return;
        }
        // Split contents into lines
        let lines = contents.split(|&byte| byte == b'\n');

        for (line_num, line_with_whitespace) in lines.enumerate() {
            let line = line_with_whitespace.trim();
            let mut matches: Vec<Match> = Vec::new();
            for (j, pattern) in CONFIG.patterns.iter().enumerate() {
                let mut it = pattern.find_iter(&line).peekable();
                if it.peek().is_none() {
                    continue;
                }
                for m in it {
                    matches.push(Match {
                        regex_id: j,
                        start: m.start(),
                        end: m.end(),
                    });
                }
            }
            if matches.len() > 0 {
                // parse through and set the overlapping to have no issues
                matches.sort_by_key(|m| m.end);

                let mut m_id = 1;
                while m_id < matches.len() {
                    // if this one starts before the previous ended
                    if matches[m_id].start < matches[m_id - 1].end {
                        // Overlap found
                        matches[m_id].start = matches[m_id - 1].end;
                    }
                    m_id += 1;
                }
                self.lines.push(LineMatch {
                    contents: line.to_vec(),
                    matches,
                    line_num: line_num + 1,
                });
            }
        }
    }
}

// Make this return a list of all directories and a list of files each
// have relative pointers to things, files don't have a reason storing relative pointers
pub fn begin_search_on_directory(root_path: PathBuf) -> Result<DirPointer, Errors> {
    let w = WalkBuilder::new(&root_path)
        .hidden(!CONFIG.search_hidden)
        .max_depth(CONFIG.max_depth)
        .build();
    // this stores every directory whether or not it has a matched file
    let mut directories: HashMap<OsString, DirPointer> = HashMap::new();
    let name = get_name_as_string(&root_path).unwrap_or_else(|_| "/".to_string());
    let top_dir = Directory::new(name, root_path);
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
                    let stringed: &OsString = &pb.as_os_str().to_os_string();
                    if directories.get(stringed).is_none() {
                        let new_dir = Directory::new(name, pb);
                        let nd_ref = Rc::new(RefCell::new(new_dir));
                        // directories.insert(&, nd_ref);
                        directories.insert(stringed.clone(), nd_ref.clone());
                    }
                } else if pb.is_file() {
                    // this returns none if file isn't text or has no matched lines
                    let m_file = search_file(pb)?;
                    // if the file had matches
                    if let Some(file) = m_file.and_then(|file| {
                        if file.lines.len() > 0 {
                            Some(file)
                        } else {
                            None
                        }
                    }) {
                        let m_dir_path: Option<PathBuf> =
                            file.path.parent().map(|v| v.to_path_buf());
                        if m_dir_path == Some(td_ref.borrow().path.clone()) {
                            // if the parent is none we are in the top directory so add it to that
                            td_ref.borrow_mut().found_files.push(file);
                        } else if let Some(dir_path) = m_dir_path {
                            // while file.parent isnt the root path
                            // we add them to a list to be
                            let mut dir_ref: &DirPointer =
                                directories.get(dir_path.as_os_str()).unwrap();

                            dir_ref.borrow_mut().found_files.push(file);
                            let mut m_dir_parent_path = dir_path.parent();
                            // and the to add check
                            while let Some(dir_parent_path) = m_dir_parent_path {
                                if dir_parent_path == &td_ref.borrow().path {
                                    break;
                                }
                                let parent_ref =
                                    directories.get(dir_parent_path.as_os_str()).unwrap();
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
            _ => {}
        }
    }
    Ok(td_ref)
}

pub fn search_file(pb: PathBuf) -> Result<Option<File>, Errors> {
    let m_content_bytes: Option<Vec<u8>> = fs::read(&pb).ok();

    let content_bytes: Vec<u8>;
    match m_content_bytes {
        None => return Ok(None),
        Some(b) => content_bytes = b,
    }

    let linked: Option<PathBuf> =
        fs::read_link(&pb)
            .ok()
            .and_then(|target_path| match std::env::var("HOME").ok() {
                Some(home) => {
                    if target_path.starts_with(&home) {
                        target_path
                            .strip_prefix(&home)
                            .ok()
                            .map(|clean_path| PathBuf::from("~").join(clean_path))
                    } else {
                        Some(target_path)
                    }
                }
                None => Some(target_path),
            });

    let mut file = File {
        lines: Vec::new(),
        name: get_name_as_string(&pb)?,
        path: pb,
        linked,
    };

    file.add_matches(content_bytes);

    return Ok(Some(file));
}

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
