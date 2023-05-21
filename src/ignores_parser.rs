// SPDX-License-Identifier: GPL-3.0

use std::collections::HashSet;
use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader};
// parse .gitignores and .ignore files to ignore the files/directories in them
fn parse_ignore_file(names: &mut HashSet<String>, path: &DirEntry) {
    // read the file contents
    let file = File::open(path.path()).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        names.insert(line.unwrap());
    }
}

pub fn parse_for_ignores(names: &mut HashSet<String>, entries: &Vec<DirEntry>) {
    for entry in entries {
        let name = entry.file_name();
        if name == ".gitignore" {
            names.insert(".git".to_string());
            parse_ignore_file(names, entry);
        }
        if name == ".ignore" {
            parse_ignore_file(names, entry);
        }
    }
}
