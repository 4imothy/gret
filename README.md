### TODO

A command line tool for code analysis. Todo recursively
search directories showing you TODO, HACK, NOTE and FIXME
instances in a tree structure.
 
![alt text](./images/example.jpg)

#### To Run
Use *cargo run* which will search on the
project's source. If you give it a directory with
*cargo run /path_to_some/some_dir* it will search that
directory instead.

#### To Install
Run the *./add_to_path.sh* after changing the
links location to somewhere on your path. Or run
the commands seperately:

```
cargo build --release
```

Then add the binary to your path and then source the
script to give you proper completions. For the completions
to work on next login you must source it at each login.

| Shell |Completion Script to Source |
|----| ---|
|BASH       |completions/todo.bash|
|Zsh        | completions/_todo|
|Fish       | completions/todo.fish|
|Elvish     |completions/todo.elv|
|PowerShell | _todo.ps1|

