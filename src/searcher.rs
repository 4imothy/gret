// SPDX-License-Identifier: Unlicense

use crate::Errors;
use crate::CONFIG;
use formats::{get_color, BOLD as BOLD_STR, LINE_NUMBER_COLOR, RESET as RESET_STR};
use ignore::WalkBuilder;
use lazy_static::lazy_static;
use memchr::memchr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub type DirPointer = Rc<RefCell<Directory>>;

lazy_static! {
    static ref BOLD: Vec<u8> = BOLD_STR.as_bytes().to_vec();
    static ref RESET: Vec<u8> = RESET_STR.as_bytes().to_vec();
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

#[derive(Debug)]
struct Match {
    matcher_id: usize,
    start: usize,
    end: usize,
}

impl File {
    fn add_matches(&mut self, contents: Vec<u8>) {
        // check if it is a binary file
        if memchr(0, &contents).is_some() {
            return;
        }
        let lines = contents.split(|&byte| byte == b'\n'); // Split contents into lines

        for (i, line) in lines.enumerate() {
            let mut matches: Vec<Match> = Vec::new();
            for (j, pattern) in CONFIG.patterns.iter().enumerate() {
                let mut it = pattern.find_iter(&line).peekable();
                if it.peek().is_none() {
                    continue;
                }
                for m in it {
                    matches.push(Match {
                        matcher_id: j,
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
                // now the matches have no overlaps
                let mut new: Vec<u8> = Vec::with_capacity(line.len());
                let mut last_match = 0;
                for m in matches {
                    new.extend_from_slice(&line[last_match..m.start]);
                    last_match = m.end;
                    if CONFIG.styled {
                        new.extend_from_slice(&get_color(m.matcher_id));
                        new.extend_from_slice(&BOLD);
                    }
                    new.extend_from_slice(&line[m.start..m.end]);
                    if CONFIG.styled {
                        new.extend_from_slice(&RESET);
                    }
                }
                new.extend_from_slice(&line[last_match..]);

                let line_to_push = if CONFIG.show_line_number {
                    let line_idx = i + 1;
                    let line_number = if CONFIG.styled {
                        format!(
                            "{}{}: {}{}",
                            LINE_NUMBER_COLOR,
                            line_idx,
                            RESET_STR,
                            String::from_utf8_lossy(&new).trim()
                        )
                    } else {
                        format!("{}: {}", line_idx, String::from_utf8_lossy(&new).trim())
                    };
                    line_number
                } else {
                    String::from_utf8_lossy(&new).trim().to_string()
                };
                self.lines.push(line_to_push);
            }
        }
    }
}

pub fn begin_search_on_directory(root_path: &PathBuf) -> Result<DirPointer, Errors> {
    // TODO make this an option
    let w = WalkBuilder::new(root_path)
        .hidden(!CONFIG.search_hidden)
        .max_depth(CONFIG.max_depth)
        .build();
    // this stores every directory whether or not it has a matched file
    let mut directories: HashMap<PathBuf, DirPointer> = HashMap::new();
    let name = get_name_as_string(root_path).unwrap_or_else(|_| "/".to_string());
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
                    let file = search_file(&pb)?;
                    // if the file had matches
                    if file.lines.len() != 0 {
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
                // TODO make this error handle better
                println!("{:?}", err);
            }
        }
    }
    Ok(td_ref)
}

pub fn search_file(pb: &PathBuf) -> Result<File, Errors> {
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
        return Ok(file);
    }

    return Ok(file);
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
