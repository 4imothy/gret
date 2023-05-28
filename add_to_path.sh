#!/bin/sh
cargo build --release
# make this something in your path
ln -sf $(pwd)/target/release/todo ~/bin/todo
