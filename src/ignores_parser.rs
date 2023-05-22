// SPDX-License-Identifier: GPL-3.0

use std::collections::HashSet;
use std::fs::{DirEntry, File};
use std::io::{self, BufRead};
use std::path::Path;
// parse .gitignores and .ignore files to ignore the files/directories in them
fn parse_ignore_file(names: &mut HashSet<String>, path: &DirEntry) {
    // read the file contents
    if let Ok(lines) = read_lines(path.path()) {
        for line in lines {
            names.insert(line.unwrap());
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
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
