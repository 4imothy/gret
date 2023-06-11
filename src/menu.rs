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

const SCROLL_OFFSET: u16 = 5;
const START_X: u16 = 1;

pub fn draw<W>(out: &mut W, searched: SearchedTypes) -> io::Result<()>
where
    W: Write,
{
    execute!(
        out,
        style::ResetColor,
        cursor::Hide,
        terminal::EnterAlternateScreen,
        // line wrapping causes issues with cursor y being off
        // from where it should be
        terminal::DisableLineWrap,
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
    let lines: Vec<String> = buffer
        .split(|&byte| byte == b'\n')
        .map(|vec| String::from_utf8_lossy(vec).into_owned())
        .collect();

    draw_loop(out, lines, searched)
}

fn print_structure<W>(out: &mut W, lines: &Vec<String>, max_prints: Option<u16>) -> io::Result<()>
where
    W: Write,
{
    queue!(out, cursor::MoveTo(START_X, 1))?;
    for (i, line) in lines
        .iter()
        .enumerate()
        .take(max_prints.map(|v| v as usize).unwrap_or(lines.len()))
    {
        if i == 0 {
            queue!(
                out,
                Print(line.as_str().on(formats::MENU_SELECTED)),
                cursor::MoveToNextLine(1)
            )?;
        } else {
            queue!(
                out,
                Print(" ".to_string() + line),
                cursor::MoveToNextLine(1)
            )?;
        }
    }
    out.flush()
}

fn draw_loop<W>(out: &mut W, lines: Vec<String>, searched: SearchedTypes) -> io::Result<()>
where
    W: Write,
{
    let mut selected: usize = 0;
    let max_selected_id: usize = lines.len() - 1;
    // - 1 on height because we start printing at (1,1)
    // size -> (columns, rows)
    // if we were able to get the size then enable the scrolling feature
    let mut max_prints: Option<u16> = terminal::size().ok().map(|(_, height)| height - 1);
    let scrolling: bool = max_prints.is_some();
    let mut cursor_y: u16 = START_X;
    print_structure(out, &lines, max_prints)?;
    'outer: loop {
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
                            destyle_selected(out, cursor_y, lines.get(selected).unwrap())?;
                            selected += 1;
                            style_selected(out, cursor_y + 1, lines.get(selected).unwrap())?;
                            if scrolling && cursor_y + SCROLL_OFFSET == max_prints.unwrap() {
                                execute!(out, terminal::ScrollUp(1))?;
                                if (selected + SCROLL_OFFSET as usize) < lines.len() {
                                    execute!(out, cursor::MoveTo(START_X, max_prints.unwrap()))?;
                                    execute!(
                                        out,
                                        Print(
                                            lines.get(selected + SCROLL_OFFSET as usize).unwrap()
                                        )
                                    )?;
                                }
                            } else {
                                cursor_y += 1;
                            }
                        }
                    }
                    'k' => {
                        if selected > 0 {
                            destyle_selected(out, cursor_y, lines.get(selected).unwrap())?;
                            selected -= 1;
                            style_selected(out, cursor_y - 1, lines.get(selected).unwrap())?;
                            if selected < SCROLL_OFFSET as usize {
                                cursor_y -= 1;
                            } else if scrolling && cursor_y == SCROLL_OFFSET {
                                execute!(out, terminal::ScrollDown(1))?;
                                if selected + 1 > SCROLL_OFFSET as usize {
                                    execute!(out, cursor::MoveTo(START_X, 0))?;
                                    execute!(
                                        out,
                                        Print(
                                            lines.get(selected - SCROLL_OFFSET as usize).unwrap()
                                        )
                                    )?;
                                }
                            } else {
                                cursor_y -= 1;
                            }
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
        } else if let Ok(Event::Resize(_, rows)) = event {
            if max_prints != Some(rows) {
                // does a overflow line cross over to the next?
                execute!(out, terminal::Clear(ClearType::All))?;
                max_prints = Some(rows);
                print_structure(out, &lines, max_prints)?;
                selected = 0;
                cursor_y = 1;
            }
        }
    }

    // TODO make a leave function so that
    // when an error occurs above still call
    // these cleanup functions
    cleanup(out)?;

    Ok(())
}

fn style_selected<W>(out: &mut W, cursor_y: u16, line: &str) -> io::Result<()>
where
    W: Write,
{
    execute!(
        out,
        cursor::MoveTo(START_X, cursor_y),
        style::Print(line.on(formats::MENU_SELECTED))
    )
}

fn destyle_selected<W>(out: &mut W, cursor_y: u16, line: &str) -> io::Result<()>
where
    W: Write,
{
    execute!(out, cursor::MoveTo(START_X, cursor_y), Print(line))
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
                "hx" => {
                    let mut arg: std::ffi::OsString = path.as_os_str().to_os_string();
                    arg.push(":");
                    arg.push(l.to_string());
                    command.arg(arg);
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
