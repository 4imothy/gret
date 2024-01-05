# NOTICE OF ARCHIVAL

*gret* is replaced by [treegrep](https://github.com/4imothy/treegrep)

Treegrep can act as a pattern matcher on its own or parse output from [ripgrep](https://github.com/BurntSushi/ripgrep) which leads to much better performance.

### gret

gret (Global Regular Expression Tree) is a command-line utility
designed to search through directories and files for a regex
expression that matches while respecting *.gitignore* and *.ignore*
files, the results are presented in a tree format and a menu can be
spawned to select from.


https://github.com/4imothy/gret/assets/40186632/07043fef-4376-433a-90a2-33c3913570dd


#### Quick Install
*cargo install gret*

#### To Run
Use *cargo run*, the first positional argument
is the pattern, the second is the path to search. If
you want to match multiple patterns use *-e* followed
by the pattern.

To launch a menu use the flag *-m* or *--menu*, this
will open a match picker. After selecting one by pression *enter*
the file/directory will be launched by *\$EDITOR* if on unix or *start*
if on windows. If *\$EDITOR* is not found, then *open* will be called on
macos and *xdg-open* will be called on other non-windows operating systems.

#### How To Use
See the [options.md](./options.md) file.

#### To Install
```
cargo install gret
```

**or**

Run the *./add_to_path.sh* script after changing the
links location to somewhere on your path. Or run
the commands seperately:
```
cargo build --release
```
And then source the correct completion file that is in the
*completions/* directory.

#### To Benchmark
Run *./benchmarks/bench* at the root directory. Results can be seen in the
*times* file in the *benchmarks* directory.

Then add the binary to your path and then source the
script to give you proper completions. For the completions
to work on next login you must source it at each login.

| Shell |Completion Script to Source |
|----| ---|
|BASH       |completions/gret.bash|
|Zsh        | completions/_gret|
|Fish       | completions/gret.fish|
|Elvish     |completions/gret.elv|
|PowerShell | _gret.ps1|
