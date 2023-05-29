#!/bin/sh
cargo build --release
# make this something in your path
ln -sf $(pwd)/target/release/todo ~/bin/todo

# Determine the type of shell
SHELL_NAME=$(basename "$SHELL")

# Source the corresponding completion file
case "$SHELL_NAME" in
  "bash")
    source completions/todo.bash
    ;;
  "zsh")
    source completions/_todo
    ;;
  "fish")
    source completions/todo.fish
    ;;
  "elvish")
    source completions/todo.elv
    ;;
  "powershell")
    source _todo.ps1
    ;;
  *)
    echo "Shell type not supported for sourcing completion file."
    ;;
esac
