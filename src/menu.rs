/*
user selects a line from a file with j,k and then
enter key or by pressing the letters/numbers on the
left side of the match, open with $EDITOR
*/

use crate::formats::MENU_SELECTED;
use crate::printer;
use crate::searcher::DirPointer;
pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, queue,
    style::{self, Attribute, Color, Print, Stylize},
    terminal::{self, ClearType},
    ErrorKind,
};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

pub fn draw<W>(out: &mut W, top_dir: DirPointer) -> io::Result<()>
where
    W: Write,
{
    execute!(
        out,
        style::ResetColor,
        cursor::Hide,
        terminal::EnterAlternateScreen
    )?;

    terminal::enable_raw_mode()?;

    // first test with just highlighting different lines
    let mut buffer: Vec<u8> = Vec::new();
    printer::start_print_directory(&mut buffer, &top_dir)?;

    let lines: Vec<String> = buffer
        .split(|&byte| byte == b'\n')
        .map(|vec| String::from_utf8_lossy(vec).into_owned())
        .collect();
    draw_loop(out, lines, top_dir)?;

    Ok(())
}

fn draw_loop<W>(out: &mut W, lines: Vec<String>, top_dir: DirPointer) -> io::Result<()>
where
    W: Write,
{
    // this will cause errors if there are more than 2^(16)-1=65535 matches
    let mut selected: u16 = 0;
    let max_selected_id = lines.len() as u16 - 1;
    let size: Option<(u16, u16)> = terminal::size().ok();
    'outer: loop {
        if size.is_some() && size != terminal::size().ok() {
            queue!(out, terminal::Clear(ClearType::All),)?;
        }

        // TODO make sure saving the style as a string works on windows
        for (i, line) in lines.iter().enumerate() {
            queue!(out, cursor::MoveTo(1, i as u16 + 1))?;
            if i as u16 == selected {
                // this only works until there is a style reset
                queue!(out, MENU_SELECTED)?;
            }
            queue!(out, Print(line))?;
        }

        out.flush()?;

        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match code {
                KeyCode::Char(c) => match c {
                    'j' => {
                        if selected < max_selected_id - 1 {
                            selected += 1
                        }
                    }
                    'k' => {
                        if selected > 0 {
                            selected -= 1
                        }
                    }
                    'q' => break 'outer,
                    'c' => {
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            break 'outer;
                        }
                    }
                    _ => {}
                },
                KeyCode::Enter => {
                    return find_selected_and_edit(out, selected, &top_dir);
                }
                _ => {}
            }
        }
    }

    // TODO make a leave function so that
    // when an error occurs above still call
    // these cleanup functions
    cleanup(out)
}

fn find_selected_and_edit<W>(out: &mut W, selected: u16, top_dir: &DirPointer) -> io::Result<()>
where
    W: Write,
{
    let mut current: u16 = 0;
    return handle_dir(out, selected, &mut current, top_dir);
}

fn handle_dir<W>(out: &mut W, selected: u16, current: &mut u16, dir: &DirPointer) -> io::Result<()>
where
    W: Write,
{
    if *current == selected {
        call_editor_exit(out, &dir.borrow().path, None)?;
    }
    *current += 1;
    for child in dir.borrow().children.iter() {
        handle_dir(out, selected, current, child)?;
    }
    for file in dir.borrow().found_files.iter() {
        if *current == selected {
            call_editor_exit(out, &file.path, None)?;
        }
        *current += 1;
        for line_match in file.lines.iter() {
            if *current == selected {
                call_editor_exit(out, &file.path, Some(line_match.line_num))?;
            }
            *current += 1;
        }
    }
    Ok(())
}

/*
Commands for editors that can open on a line number, this is only active for
searches that include the line number other wise just $EDITOR:
`n` is the line number
- vi: +n file
- vim: +n file
- neovim: +n file
- vscode (code): --goto file:n
- nano: +n file
*/
fn call_editor_exit<W>(out: &mut W, path: &PathBuf, line_num: Option<usize>) -> io::Result<()>
where
    W: Write,
{
    let opener: String;
    #[cfg(not(windows))]
    {
        opener = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    }
    #[cfg(windows)]
    {
        opener = "start".to_string();
    }
    let mut args: Vec<String> = Vec::new();
    if let Some(l) = line_num {
        match opener.as_str() {
            "vi" | "vim" | "nvim" | "nano" => args.push(format!("+{}", l)),
            "code" => args.push(format!("--goto file:{}", l)),
            _ => {}
        }
    };

    #[cfg(not(windows))]
    {
        use std::os::unix::process::CommandExt;
        cleanup(out)?;
        Command::new(opener).arg(path).args(args).exec();
    }

    Ok(())
}

fn cleanup<W>(out: &mut W) -> io::Result<()>
where
    W: Write,
{
    terminal::disable_raw_mode()?;
    execute!(
        out,
        style::ResetColor,
        cursor::SetCursorStyle::DefaultUserShape,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    Ok(())
}
