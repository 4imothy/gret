complete -c gret -s e -d 'specify the regex expression' -r
complete -c gret -s t -l target -d 'specify the search target. If none provided, search the current directory.' -r -F
complete -c gret -s m -l max_depth -d 'the max depth the searcher will search' -r
complete -c gret -s b -l bland -d 'if this option is present there will be no styling of text'
complete -c gret -s c -l show_count -d 'if this option is present, display number of files matched in a directory and number of lines matched in a file'
complete -c gret -s a -l hidden -d 'if this option is present gret will search hidden files'
complete -c gret -s l -l line_number -d 'if this option is present show the line number of the matched text'
complete -c gret -s h -l help -d 'Print help'
