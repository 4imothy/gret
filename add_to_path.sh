#!/bin/sh
cargo build --release
# make this something in your path
ln -sf $(pwd)/target/release/gret ~/bin/gret

# Determine the type of shell
SHELL_NAME=$(basename "$SHELL")

# Source the corresponding completion file
case "$SHELL_NAME" in
  "bash")
    source completions/gret.bash
    ;;
  "zsh")
    source completions/_gret
    ;;
  "fish")
    source completions/gret.fish
    ;;
  "elvish")
    source completions/gret.elv
    ;;
  "powershell")
    source _gret.ps1
    ;;
  *)
    echo "Shell type not supported for sourcing completion file."
    ;;
esac
