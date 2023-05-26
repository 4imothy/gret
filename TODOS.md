### TODO
#### Ignore
- Implement globs
- gitignore fix for starting paths with '/', it is relative to the parent directory, .DS_Store isn't ignored because it is in ignore file it is ignored because it is binary which is wrong

#### Optimizations
- Can just call is_file on a path_buf to check if a symlinks linked file exists
#### Bugs
- Why is ignore_this.txt not being found
#### New Features
- Better formatting for printing
- Add a counter for the number of files, number of children, etc.
- Make a UI, list out the todos and select one then enter the file
- Only read comments in files maybe?
#### Other
- Check running on windows
- Bench againts popular cli search tools
