/*
user selects a line from a file with j,k and then
enter key or by pressing the letters/numbers on the
left side of the match, open with $EDITOR
*/

use crate::formats;
use crate::printer;
use crate::searcher::{DirPointer, File};
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

pub enum SearchedTypes {
    Dir(DirPointer),
    File(File),
}

const SCROLL_OFFSET: usize = 5;

pub fn draw<W>(out: &mut W, searched: SearchedTypes) -> io::Result<()>
where
    W: Write,
{
    execute!(
        out,
        style::ResetColor,
        cursor::Hide,
        terminal::EnterAlternateScreen,
        terminal::EnableLineWrap,
    )?;
    terminal::enable_raw_mode()?;

    let mut buffer: Vec<u8> = Vec::new();
    match &searched {
        SearchedTypes::Dir(dir) => {
            printer::start_print_directory(&mut buffer, &dir)?;
        }
        SearchedTypes::File(file) => {
            printer::print_single_file(&mut buffer, &file)?;
        }
    }

    // first test with just highlighting different lines

    let lines: Vec<String> = buffer
        .split(|&byte| byte == b'\n')
        .map(|vec| String::from_utf8_lossy(vec).into_owned())
        .collect();

    draw_loop(out, lines, searched)
}

fn draw_loop<W>(out: &mut W, lines: Vec<String>, searched: SearchedTypes) -> io::Result<()>
where
    W: Write,
{
    let mut selected: usize = 0;
    let max_selected_id = lines.len() - 1;
    // (columns, rows)
    let mut size: Option<(u16, u16)> = terminal::size().ok();
    // - 1 on height because we start printing at (1,1)
    let mut max_prints: Option<usize> = size.map(|(_, height)| height as usize - 1);
    // TODO make this an option tuple to clean up
    let mut window_start: Option<usize> = Some(0);
    let mut window_end: Option<usize> = match (window_start, max_prints) {
        (Some(start), Some(max)) => Some(start + max),
        _ => None,
    };
    'outer: loop {
        queue!(out, cursor::MoveTo(1, 1))?;

        // selected % (max_prints + 1) tells the
        // number lines down that selected is
        // probably better to use the crossterm::ScrollUp But I
        // am not sure how to implement that
        if let (Some(end), Some(start)) = (window_end, window_start) {
            if selected + SCROLL_OFFSET == end {
                window_start = Some(start + 1);
                window_end = Some(end + 1);
                execute!(out, terminal::Clear(ClearType::All))?;
            }
            if selected + 1 > SCROLL_OFFSET && selected + 1 - SCROLL_OFFSET == start {
                window_start = Some(start - 1);
                window_end = Some(end - 1);
                execute!(out, terminal::Clear(ClearType::All))?;
            }
        }

        for (i, line) in lines
            .iter()
            .enumerate()
            .skip(window_start.unwrap_or(0))
            .take(max_prints.unwrap_or(lines.len()))
        {
            if i == selected {
                queue!(out, formats::MENU_SELECTED)?;
            }
            queue!(out, Print(line), cursor::MoveToNextLine(1))?;
        }

        out.flush()?;

        let event = event::read();

        if let Ok(Event::Key(KeyEvent {
            code,
            modifiers,
            kind: crossterm::event::KeyEventKind::Press,
            ..
        })) = event
        {
            match code {
                KeyCode::Char(c) => match c {
                    'j' => {
                        if selected < max_selected_id - 1 {
                            selected += 1;
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
                    return find_selected_and_edit(out, selected, searched);
                }
                _ => {}
            }
        } else if let Ok(Event::Resize(cols, rows)) = event {
            if size.is_some() && size != terminal::size().ok() {
                queue!(out, terminal::Clear(ClearType::All))?;
                size = Some((cols, rows));
                max_prints = size.map(|(_, height)| height as usize);
                window_end = match (window_start, max_prints) {
                    (Some(start), Some(max)) => Some(start + max),
                    _ => None,
                };
            }
        }
    }

    // TODO make a leave function so that
    // when an error occurs above still call
    // these cleanup functions
    cleanup(out)?;

    Ok(())
}

fn find_selected_and_edit<W>(
    out: &mut W,
    selected: usize,
    searched: SearchedTypes,
) -> io::Result<()>
where
    W: Write,
{
    let mut current: usize = 0;
    match searched {
        SearchedTypes::Dir(dir) => {
            return handle_dir(out, selected, &mut current, &dir);
        }
        SearchedTypes::File(file) => {
            return handle_file(out, selected, &mut current, &file);
        }
    }
}

fn handle_file<W>(out: &mut W, selected: usize, current: &mut usize, file: &File) -> io::Result<()>
where
    W: Write,
{
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

    Ok(())
}

fn handle_dir<W>(
    out: &mut W,
    selected: usize,
    current: &mut usize,
    dir: &DirPointer,
) -> io::Result<()>
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
    #[cfg(not(windows))]
    {
        let mut opener: String = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        // if the env var isn't set than use open on macos xdg-open on other
        if opener.is_empty() {
            opener = match std::env::consts::OS {
                "macos" => "open".to_string(),
                _ => "xdg-open".to_string(),
            };
        }
        let mut command: Command = Command::new(opener.clone());
        if let Some(l) = line_num {
            match opener.as_str() {
                "vi" | "vim" | "nvim" | "nano" => {
                    command.arg(format!("+{l}"));
                    command.arg(path);
                }
                "code" => {
                    command.arg("--goto");
                    let mut arg: std::ffi::OsString = path.as_os_str().to_os_string();
                    arg.push(":");
                    arg.push(l.to_string());
                    command.arg(arg);
                }
                _ => {
                    command.arg(path);
                }
            }
        } else {
            command.arg(path);
        }

        use std::os::unix::process::CommandExt;
        // don't leave alt screen here to avoid crash
        cleanup(out)?;
        command.exec();
    }

    #[cfg(windows)]
    {
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg(path)
            .spawn()?;
        cleanup(out)?;
    }

    Ok(())
}

fn cleanup<W>(out: &mut W) -> io::Result<()>
where
    W: Write,
{
    // don't leave alt screen here to avoid flashing the
    // normal shell screen
    terminal::disable_raw_mode()?;
    execute!(
        out,
        style::ResetColor,
        cursor::SetCursorStyle::DefaultUserShape,
        terminal::LeaveAlternateScreen, // this will flash the normal prompt
        cursor::Show,
    )
}
