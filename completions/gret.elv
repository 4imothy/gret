
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
            cand -e 'Specify the regex expression'
            cand -t 'Specify the search target. If none provided, search the current directory.'
            cand --target 'Specify the search target. If none provided, search the current directory.'
            cand -b 'Whether to style output'
            cand --bland 'Whether to style output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
    ]
    $completions[$command]
}
