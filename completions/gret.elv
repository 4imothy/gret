
use builtin;
use str;

set edit:completion:arg-completer[gret] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'gret'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'gret'= {
            cand -e 'specify the regex expression'
            cand --expr 'specify the regex expression'
            cand -t 'specify the search target. If none provided, search the current directory.'
            cand --target 'specify the search target. If none provided, search the current directory.'
            cand --max_depth 'the max depth the searcher will search'
            cand -b 'if this option is present there will be no styling of text'
            cand --bland 'if this option is present there will be no styling of text'
            cand -c 'if this option is present, display number of files matched in a directory and number of lines matched in a file'
            cand --show_count 'if this option is present, display number of files matched in a directory and number of lines matched in a file'
            cand -a 'if this option is present gret will search hidden files'
            cand --hidden 'if this option is present gret will search hidden files'
            cand -l 'if this option is present show the line number of the matched text'
            cand --line_number 'if this option is present show the line number of the matched text'
            cand -m 'if this arg is present gret will show matches in a menu to be selected from'
            cand --menu 'if this arg is present gret will show matches in a menu to be selected from'
            cand -h 'Print help'
            cand --help 'Print help'
        }
    ]
    $completions[$command]
}
