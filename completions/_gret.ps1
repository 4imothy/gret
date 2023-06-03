
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'gret' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'gret'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'gret' {
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'specify the regex expression')
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'specify the search target. If none provided, search the current directory.')
            [CompletionResult]::new('--target', 'target', [CompletionResultType]::ParameterName, 'specify the search target. If none provided, search the current directory.')
            [CompletionResult]::new('-m', 'm', [CompletionResultType]::ParameterName, 'the max depth the searcher will search')
            [CompletionResult]::new('--max_depth', 'max_depth', [CompletionResultType]::ParameterName, 'the max depth the searcher will search')
            [CompletionResult]::new('-b', 'b', [CompletionResultType]::ParameterName, 'if this option is present there will be no styling of text')
            [CompletionResult]::new('--bland', 'bland', [CompletionResultType]::ParameterName, 'if this option is present there will be no styling of text')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'if this option is present, display number of files matched in a directory and number of lines matched in a file')
            [CompletionResult]::new('--show_count', 'show_count', [CompletionResultType]::ParameterName, 'if this option is present, display number of files matched in a directory and number of lines matched in a file')
            [CompletionResult]::new('-a', 'a', [CompletionResultType]::ParameterName, 'if this option is present gret will search hidden files')
            [CompletionResult]::new('--hidden', 'hidden', [CompletionResultType]::ParameterName, 'if this option is present gret will search hidden files')
            [CompletionResult]::new('-l', 'l', [CompletionResultType]::ParameterName, 'if this option is present show the line number of the matched text')
            [CompletionResult]::new('--line_number', 'line_number', [CompletionResultType]::ParameterName, 'if this option is present show the line number of the matched text')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
