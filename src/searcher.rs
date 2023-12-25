// SPDX-License-Identifier: CC-BY-4.0

use crate::Errors;
use crate::CONFIG;
use ignore::WalkBuilder;
use memchr::memchr;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

pub struct Directory {
    pub name: String,
    pub children: Vec<usize>,
    pub files: Vec<File>,
    pub path: OsString,
    to_add: bool,
}

pub struct File {
    pub name: String,
    pub lines: Vec<MatchedLine>,
    pub linked: Option<PathBuf>,
    pub path: PathBuf,
}

pub enum Searched {
    Dir(Vec<Directory>),
    File(File),
}

pub struct Match {
    pub regex_id: usize,
    pub start: usize,
    pub end: usize,
}

pub struct MatchedLine {
    pub line_num: usize,
    pub contents: Vec<u8>,
    pub matches: Vec<Match>,
}

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
                self.lines.push(MatchedLine {
                    contents: line.to_vec(),
                    matches,
                    line_num: line_num + 1,
                });
            }
        }
    }
}

impl Directory {
    fn new(name: String, path: OsString) -> Directory {
        Directory {
            files: Vec::new(),
            children: Vec::new(),
            to_add: true,
            name,
            path,
        }
    }
}

pub fn search_dir(root_path: PathBuf) -> Result<Vec<Directory>, Errors> {
    let walker = WalkBuilder::new(&root_path)
        .hidden(!CONFIG.search_hidden)
        .max_depth(CONFIG.max_depth)
        .build();

    let mut path_to_index: HashMap<OsString, usize> = HashMap::new();
    let mut directories: Vec<Directory> = Vec::new();
    for res in walker {
        match res {
            Ok(entry) => {
                let path = entry.into_path();
                if path.is_dir() {
                    let name: String = path_name(&path)?;
                    if path_to_index.get(path.as_os_str()).is_none() {
                        path_to_index.insert(path.clone().into_os_string(), directories.len());
                        let dir = Directory::new(name, path.into_os_string());
                        directories.push(dir);
                    }
                } else if path.is_file() {
                    let m_file = search_file(path)?;
                    if let Some(file) = m_file.and_then(|file| {
                        if file.lines.len() > 0 {
                            Some(file)
                        } else {
                            None
                        }
                    }) {
                        if let Some(mut dir_path) = file.path.parent().map(|v| v.to_path_buf()) {
                            let mut prev_id: usize =
                                *path_to_index.get(dir_path.as_os_str()).unwrap();
                            let mut dir: &mut Directory = directories.get_mut(prev_id).unwrap();
                            dir.files.push(file);
                            let mut to_add = dir.to_add;
                            while let Some(par_dir_path) = dir_path.parent() {
                                if !to_add || dir_path == root_path {
                                    break;
                                }
                                dir.to_add = false;
                                let t = *path_to_index.get(par_dir_path.as_os_str()).unwrap();
                                dir = directories.get_mut(t).unwrap();
                                dir.children.push(prev_id);
                                prev_id = t;
                                to_add = dir.to_add;
                                dir_path = par_dir_path.to_path_buf();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(directories)
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
        name: path_name(&pb)?,
        path: pb,
        linked,
    };

    file.add_matches(content_bytes);

    return Ok(Some(file));
}

fn path_name(path: &PathBuf) -> Result<String, Errors> {
    let name = path.file_name().ok_or(Errors::CantGetName {
        cause: path.to_path_buf(),
    })?;

    name.to_os_string()
        .into_string()
        .map_err(|_| Errors::CantGetName {
            cause: path.to_path_buf(),
        })
}
