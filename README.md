### gret

gret (Global Regular Expression Tree) is a command-line utility
designed to search through directories and files for a regex
expression that matches while respecting *.gitignore* and *.ignore*
files, the results are presented in a tree format.


https://github.com/4imothy/gret/assets/40186632/07043fef-4376-433a-90a2-33c3913570dd


#### Quick Install
*cargo install gret*

#### To Run
Use *cargo run*, the first positional argument
is the pattern, the second is the path to search. If
you want to match multiple patterns use *-e* followed
by the pattern.

#### How To Use
See the [options.md](./options.md) file.

#### To Install
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
