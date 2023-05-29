use clap::ValueEnum;
use clap_complete::{generate_to, Shell};
use std::env;
use std::io::Error;

include!("src/command.rs");

fn main() -> Result<(), Error> {
    let _outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    // BASH -> completions/todo.bash
    // Zsh -> completions/_todo
    // Fish -> completions/todo.fish
    // Elvish -> completions/todo.elv
    // PowerShell -> _todo.ps1
    let mut cmd = generate_command();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "todo", "completions")?;
    }

    Ok(())
}
