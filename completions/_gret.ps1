
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
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'Specify the regex expression')
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'Specify the search target. If none provided, search the current directory.')
            [CompletionResult]::new('--target', 'target', [CompletionResultType]::ParameterName, 'Specify the search target. If none provided, search the current directory.')
            [CompletionResult]::new('-b', 'b', [CompletionResultType]::ParameterName, 'Whether to style output')
            [CompletionResult]::new('--bland', 'bland', [CompletionResultType]::ParameterName, 'Whether to style output')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
