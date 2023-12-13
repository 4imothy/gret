cargo build --release

# move somewhere in your path
ln -sf $(pwd)/target/release/gret ~/bin/gret

SHELL_NAME=$(basename "$SHELL")

# move the completion file to somewhere your shell looks
case "$SHELL_NAME" in
    "zsh")
        ln -sf $(pwd)/completions/_gret $ZDOTDIR/completions/_gret
        ;;
esac
