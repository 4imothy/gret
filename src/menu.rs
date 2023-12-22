// SPDX-License-Identifier: CC-BY-4.0

use crate::formats;
use crate::printer::write_results;
use crate::searcher::{DirPointer, File, SearchedTypes};
use crate::CONFIG;
pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, queue,
    style::{self, Attribute, Color, Print, Stylize},
    terminal::{self, ClearType},
};
use std::io::{self, StdoutLock, Write};
use std::path::PathBuf;
use std::process::Command;

const SCROLL_OFFSET: u16 = 5;
const START_X: u16 = 0;
const START_Y: u16 = 0;

pub fn draw(out: &mut impl Write, searched: SearchedTypes) -> io::Result<()> {
    enter(out)?;
    let mut buffer: Vec<u8> = Vec::new();
    write_results(&mut buffer, &searched)?;
    let lines: Vec<String> = buffer
        .split(|&byte| byte == b'\n')
        .map(|vec| String::from_utf8_lossy(vec).into_owned())
        .collect();

    draw_loop(out, lines, searched)
}

fn print_structure(out: &mut impl Write, lines: &Vec<String>, num_rows: u16) -> io::Result<()> {
    queue!(out, cursor::MoveTo(START_X, START_Y))?;
    for (i, line) in lines.iter().enumerate().take(num_rows as usize) {
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

fn draw_loop(out: &mut impl Write, lines: Vec<String>, searched: SearchedTypes) -> io::Result<()> {
    let mut selected: usize = 0;
    let max_selected_id: usize = lines.len() - 1;
    let mut num_rows: u16 = terminal::size()
        .ok()
        .map(|(_, height)| height)
        .unwrap_or_else(|| {
            if lines.len() > i16::max_value() as usize {
                u16::max_value()
            } else {
                lines.len() as u16
            }
        });
    let mut cursor_y: u16 = START_Y;
    print_structure(out, &lines, num_rows)?;
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
                            move_down(out, &mut selected, &mut cursor_y, num_rows, &lines)?;
                        }
                    }
                    'k' => {
                        if selected > 0 {
                            move_up(out, &mut selected, &mut cursor_y, &lines)?;
                        }
                    }
                    'q' => break 'outer,
                    'c' => {
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            break 'outer;
                        }
                    }
                    'z' => {
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            suspend(out)?;
                            resume(out, &mut num_rows, &mut selected, &mut cursor_y, &lines)?;
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
            if num_rows != rows {
                num_rows = rows;
                redraw(out, num_rows, &lines, &mut selected, &mut cursor_y)?;
            }
        }
    }

    leave(out)?;

    Ok(())
}

fn suspend(out: &mut impl Write) -> io::Result<()> {
    #[cfg(not(windows))]
    {
        leave(out)?;
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP).unwrap();
    }
    Ok(())
}

fn resume(
    out: &mut impl Write,
    num_rows: &mut u16,
    selected: &mut usize,
    cursor_y: &mut u16,
    lines: &Vec<String>,
) -> io::Result<()> {
    *num_rows = terminal::size()
        .ok()
        .map(|(_, height)| height)
        .unwrap_or_else(|| {
            if lines.len() > i16::max_value() as usize {
                u16::max_value()
            } else {
                lines.len() as u16
            }
        });
    enter(out)?;
    redraw(out, *num_rows, &lines, selected, cursor_y)?;
    Ok(())
}

// TODO make this work with keeping the selected id
fn redraw(
    out: &mut impl Write,
    num_rows: u16,
    lines: &Vec<String>,
    selected: &mut usize,
    cursor_y: &mut u16,
) -> io::Result<()> {
    execute!(out, terminal::Clear(ClearType::All))?;
    print_structure(out, &lines, num_rows)?;
    *selected = 0;
    *cursor_y = START_Y;
    Ok(())
}

fn move_down(
    out: &mut impl Write,
    selected: &mut usize,
    cursor_y: &mut u16,
    num_rows: u16,
    lines: &Vec<String>,
) -> io::Result<()> {
    destyle_at_cursor(out, *cursor_y, lines.get(*selected).unwrap())?;
    *selected += 1;
    style_at_cursor(out, *cursor_y + 1, lines.get(*selected).unwrap())?;
    if *selected + (SCROLL_OFFSET as usize) < num_rows as usize {
        *cursor_y += 1;
    } else if *cursor_y + SCROLL_OFFSET == num_rows {
        execute!(out, terminal::ScrollUp(1))?;
        if (*selected + SCROLL_OFFSET as usize) < lines.len() {
            execute!(out, cursor::MoveTo(START_X, num_rows))?;
            execute!(
                out,
                Print(lines.get(*selected - 1 + SCROLL_OFFSET as usize).unwrap())
            )?;
        }
    } else {
        *cursor_y += 1;
    }
    Ok(())
}

fn move_up(
    out: &mut impl Write,
    selected: &mut usize,
    cursor_y: &mut u16,
    lines: &Vec<String>,
) -> io::Result<()> {
    destyle_at_cursor(out, *cursor_y, lines.get(*selected).unwrap())?;
    *selected -= 1;
    style_at_cursor(out, *cursor_y - 1, lines.get(*selected).unwrap())?;
    if *selected < SCROLL_OFFSET as usize {
        *cursor_y -= 1;
    } else if *cursor_y == SCROLL_OFFSET {
        execute!(out, terminal::ScrollDown(1))?;
        if *selected + 1 > SCROLL_OFFSET as usize {
            execute!(out, cursor::MoveTo(START_X, START_Y))?;
            execute!(
                out,
                Print(lines.get(*selected - SCROLL_OFFSET as usize).unwrap())
            )?;
        }
    } else {
        *cursor_y -= 1;
    }

    Ok(())
}

fn style_at_cursor(out: &mut impl Write, cursor_y: u16, line: &str) -> io::Result<()> {
    execute!(
        out,
        cursor::MoveTo(START_X, cursor_y),
        style::Print(line.on(formats::MENU_SELECTED))
    )
}

fn destyle_at_cursor(out: &mut impl Write, cursor_y: u16, line: &str) -> io::Result<()> {
    execute!(out, cursor::MoveTo(START_X, cursor_y), Print(line))
}

// this logic should be easy with doing both just_files and directories
fn find_selected_and_edit(
    out: &mut impl Write,
    selected: usize,
    searched: SearchedTypes,
) -> io::Result<()> {
    if CONFIG.just_files {
        return find_selected_just_files(out, selected, searched);
    }
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

fn handle_file(
    out: &mut impl Write,
    selected: usize,
    current: &mut usize,
    file: &File,
) -> io::Result<()> {
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

fn handle_dir(
    out: &mut impl Write,
    selected: usize,
    current: &mut usize,
    dir: &DirPointer,
) -> io::Result<()> {
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

fn call_editor_exit(
    out: &mut impl Write,
    path: &PathBuf,
    line_num: Option<usize>,
) -> io::Result<()> {
    #[cfg(not(windows))]
    {
        let opener = match std::env::var("EDITOR") {
            Ok(val) if !val.is_empty() => val,
            _ => match std::env::consts::OS {
                "macos" => "open".to_string(),
                _ => "xdg-open".to_string(),
            },
        };

        let mut command: Command = Command::new(&opener);
        if let Some(l) = line_num {
            match opener.as_str() {
                "vi" | "vim" | "nvim" | "nano" | "emacs" => {
                    command.arg(format!("+{l}"));
                    command.arg(path);
                }
                "hx" => {
                    command.arg(format!("{}:{l}", path.display()));
                }
                "code" => {
                    command.arg("--goto");
                    command.arg(format!("{}:{l}", path.display()));
                }
                _ => {
                    command.arg(path);
                }
            }
        } else {
            command.arg(path);
        }

        use std::os::unix::process::CommandExt;
        leave(out)?;
        command.exec();
    }

    #[cfg(windows)]
    {
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg(path)
            .spawn()?;
        leave(out)?;
    }

    Ok(())
}

fn find_selected_just_files(
    out: &mut impl Write,
    selected: usize,
    searched: SearchedTypes,
) -> io::Result<()> {
    let mut current: usize = 0;
    match &searched {
        SearchedTypes::Dir(dir) => {
            return handle_dir_just_files(out, selected, &mut current, dir);
        }
        SearchedTypes::File(file) => {
            return call_editor_exit(out, &file.path, None);
        }
    }
}

fn handle_dir_just_files(
    out: &mut impl Write,
    selected: usize,
    current: &mut usize,
    dir_ptr: &DirPointer,
) -> io::Result<()> {
    let dir = dir_ptr.borrow();
    let children = &dir.children;
    let files = &dir.found_files;
    for child in children {
        handle_dir_just_files(out, selected, current, &child)?;
    }
    for file in files {
        if *current == selected {
            return call_editor_exit(out, &file.path, None);
        }
        *current += 1;
    }

    Ok(())
}

fn enter(out: &mut impl Write) -> io::Result<()> {
    execute!(
        out,
        style::ResetColor,
        cursor::Hide,
        terminal::EnterAlternateScreen,
        terminal::DisableLineWrap,
    )?;
    terminal::enable_raw_mode()
}

fn leave(out: &mut impl Write) -> io::Result<()> {
    terminal::disable_raw_mode()?;
    out.flush()?;
    execute!(
        io::stderr(),
        style::ResetColor,
        cursor::SetCursorStyle::DefaultUserShape,
        terminal::LeaveAlternateScreen,
        cursor::Show,
    )
}
