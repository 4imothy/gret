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
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

const SCROLL_OFFSET: u16 = 5;
const START_X: u16 = 0;
const START_Y: u16 = 0;

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
    write_results(&mut buffer, &searched)?;
    let lines: Vec<String> = buffer
        .split(|&byte| byte == b'\n')
        .map(|vec| String::from_utf8_lossy(vec).into_owned())
        .collect();

    draw_loop(out, lines, searched)
}

fn suspend<W>(out: &mut W) -> io::Result<()>
where
    W: Write,
{
    #[cfg(not(windows))]
    {
        out.flush()?;
        terminal::disable_raw_mode()?;
        execute!(
            std::io::stderr(),
            terminal::LeaveAlternateScreen,
            cursor::Show
        )?;
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP).unwrap();
    }
    Ok(())
}

fn print_structure<W>(out: &mut W, lines: &Vec<String>, max_prints: u16) -> io::Result<()>
where
    W: Write,
{
    queue!(out, cursor::MoveTo(START_X, START_Y))?;
    for (i, line) in lines.iter().enumerate().take(max_prints as usize) {
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
    let mut max_prints: u16 = terminal::size()
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
                            move_down(out, &mut selected, &mut cursor_y, max_prints, &lines)?;
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
                            max_prints = terminal::size()
                                .ok()
                                .map(|(_, height)| height)
                                .unwrap_or_else(|| {
                                    if lines.len() > i16::max_value() as usize {
                                        u16::max_value()
                                    } else {
                                        lines.len() as u16
                                    }
                                });
                            suspend(out)?;
                            terminal::enable_raw_mode()?;
                            execute!(out, terminal::EnterAlternateScreen)?;
                            redraw(out, max_prints, &lines, &mut selected, &mut cursor_y)?;
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
            if max_prints != rows {
                max_prints = rows;
                redraw(out, max_prints, &lines, &mut selected, &mut cursor_y)?;
            }
        }
    }

    // TODO make a leave function so that
    // when an error occurs above still call
    // these cleanup functions
    cleanup(out)?;

    Ok(())
}

// TODO make this work with keeping the selected id
fn redraw<W>(
    out: &mut W,
    max_prints: u16,
    lines: &Vec<String>,
    selected: &mut usize,
    cursor_y: &mut u16,
) -> io::Result<()>
where
    W: Write,
{
    execute!(out, terminal::Clear(ClearType::All))?;
    print_structure(out, &lines, max_prints)?;
    *selected = 0;
    *cursor_y = START_Y;
    Ok(())
}

fn move_down<W>(
    out: &mut W,
    selected: &mut usize,
    cursor_y: &mut u16,
    max_prints: u16,
    lines: &Vec<String>,
) -> io::Result<()>
where
    W: Write,
{
    destyle_selected(out, *cursor_y, lines.get(*selected).unwrap())?;
    *selected += 1;
    style_selected(out, *cursor_y + 1, lines.get(*selected).unwrap())?;
    if *selected + (SCROLL_OFFSET as usize) < max_prints as usize {
        *cursor_y += 1;
    } else if *cursor_y + SCROLL_OFFSET == max_prints {
        execute!(out, terminal::ScrollUp(1))?;
        if (*selected + SCROLL_OFFSET as usize) < lines.len() {
            execute!(out, cursor::MoveTo(START_X, max_prints))?;
            execute!(
                out,
                Print(lines.get(*selected + SCROLL_OFFSET as usize).unwrap())
            )?;
        }
    } else {
        *cursor_y += 1;
    }
    Ok(())
}

fn move_up<W>(
    out: &mut W,
    selected: &mut usize,
    cursor_y: &mut u16,
    lines: &Vec<String>,
) -> io::Result<()>
where
    W: Write,
{
    destyle_selected(out, *cursor_y, lines.get(*selected).unwrap())?;
    *selected -= 1;
    style_selected(out, *cursor_y - 1, lines.get(*selected).unwrap())?;
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

fn call_editor_exit<W>(out: &mut W, path: &PathBuf, line_num: Option<usize>) -> io::Result<()>
where
    W: Write,
{
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

fn find_selected_just_files<W>(
    out: &mut W,
    selected: usize,
    searched: SearchedTypes,
) -> io::Result<()>
where
    W: Write,
{
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

fn handle_dir_just_files<W>(
    out: &mut W,
    selected: usize,
    current: &mut usize,
    dir_ptr: &DirPointer,
) -> io::Result<()>
where
    W: Write,
{
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

fn cleanup<W>(out: &mut W) -> io::Result<()>
where
    W: Write,
{
    terminal::disable_raw_mode()?;
    execute!(
        out,
        style::ResetColor,
        cursor::SetCursorStyle::DefaultUserShape,
        terminal::LeaveAlternateScreen,
        cursor::Show,
    )
}
