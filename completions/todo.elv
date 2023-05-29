
use builtin;
use str;

set edit:completion:arg-completer[todo] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'todo'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'todo'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
    ]
    $completions[$command]
}
