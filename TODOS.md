### TODO

#### Bugs
- Completions on zsh (at least) display two rows of what is to be set, should just show one
- On highlighting for the menu had to overwrite the default fg to be white so that the background styling wouldn't disappear after a `RESET_COLOR` was called
- Weird printing of overlapping lines, sometimes when resizing

#### Optimizations
- Make reading a file faster

#### New Features
- Make a side bar for the menu that has numbers/letters corresponding with each row if one of those keys is pressed than enter that file
- Make work for stdin, not sure how to work with branching
