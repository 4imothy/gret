cargo build --release

ln -sf $(pwd)/target/release/gret ~/bin/gret
ln -sf $(pwd)/completions/_gret $ZDOTDIR/completions/_gret
