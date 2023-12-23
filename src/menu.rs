// SPDX-License-Identifier: CC-BY-4.0

use crate::formats;
use crate::printer;
use crate::searcher::{DirPointer, File, SearchedTypes};
use crate::CONFIG;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Print, Stylize},
    terminal::{self, ClearType},
};
use std::io::{self, StdoutLock, Write};
use std::path::PathBuf;
use std::process::Command;

const SCROLL_OFFSET: u16 = 5;
const START_X: u16 = 0;
const START_Y: u16 = 0;

// need to store the path and the line number?
// TODO can actually always search for line num just set it to be 0 and it will be the same
// TODO get rid of all the path clones this will be done with the rewrite without RefCell
struct Selected {
    path: PathBuf,
    line: usize,
}

impl Selected {
    pub fn new(path: PathBuf, line: usize) -> Selected {
        Selected { path, line }
    }

    fn get_selected_info(selected: usize, searched: &SearchedTypes) -> Selected {
        let mut current: usize = 0;
        match searched {
            SearchedTypes::Dir(dir) => {
                return Selected::search_dir(dir, selected, &mut current).unwrap();
            }
            SearchedTypes::File(file) => {
                return Selected::search_file(file, selected, &mut current).unwrap();
            }
        }
    }

    fn search_dir(dir_ptr: &DirPointer, selected: usize, current: &mut usize) -> Option<Selected> {
        let dir = dir_ptr.borrow();
        let children = &dir.children;
        let files = &dir.found_files;
        let mut sel: Option<Selected>;
        if CONFIG.just_files {
            for child in children {
                sel = Selected::search_dir(&child, selected, current);
                if sel.is_some() {
                    return sel;
                }
            }
            for file in files {
                // TODO put this in the search_file function
                sel = Selected::search_file(file, selected, current);
                if sel.is_some() {
                    return sel;
                }
            }
        } else {
            if *current == selected {
                return Some(Selected::new(dir.path.clone(), 0));
            }
            *current += 1;
            for child in children {
                sel = Selected::search_dir(&child, selected, current);
                if sel.is_some() {
                    return sel;
                }
            }
            for file in files {
                sel = Selected::search_file(file, selected, current);
                if sel.is_some() {
                    return sel;
                }
            }
        }
        return None;
    }

    fn search_file(file: &File, selected: usize, current: &mut usize) -> Option<Selected> {
        if *current == selected {
            return Some(Selected::new(file.path.clone(), 0));
        }
        *current += 1;
        if !CONFIG.just_files {
            for line in file.lines.iter() {
                if *current == selected {
                    return Some(Selected::new(file.path.clone(), line.line_num));
                }
                *current += 1;
            }
        }
        None
    }
}

pub struct Menu<'a, 'b> {
    selected_id: usize,
    cursor_y: u16,
    out: &'a mut StdoutLock<'b>,
    searched: SearchedTypes,
    lines: Vec<String>,
    num_rows: u16,
}

impl<'a, 'b> Menu<'a, 'b> {
    fn new(out: &'a mut StdoutLock<'b>, searched: SearchedTypes) -> io::Result<Menu<'a, 'b>> {
        let mut buffer: Vec<u8> = Vec::new();
        printer::write_results(&mut buffer, &searched)?;
        let lines: Vec<String> = buffer
            .split(|&byte| byte == b'\n')
            .map(|vec| String::from_utf8_lossy(vec).into_owned())
            .collect();
        Ok(Menu {
            selected_id: 0,
            cursor_y: 0,
            out,
            searched,
            lines,
            num_rows: Menu::num_rows(),
        })
    }

    fn num_rows() -> u16 {
        terminal::size().ok().map(|(_, height)| height).unwrap()
    }

    pub fn draw(out: &'a mut StdoutLock<'b>, searched: SearchedTypes) -> io::Result<()> {
        let mut menu: Menu = Menu::new(out, searched)?;

        menu.enter()?;

        let max_selected_id: usize = menu.lines.len() - 1;
        menu.write_menu()?;
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
                            if menu.selected_id < max_selected_id - 1 {
                                menu.move_down()?;
                            }
                        }
                        'k' => {
                            if menu.selected_id > 0 {
                                menu.move_up()?;
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
                                menu.suspend()?;
                                menu.resume()?;
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Enter => {
                        return menu.exit_and_open(Selected::get_selected_info(
                            menu.selected_id,
                            &menu.searched,
                        ));
                    }
                    _ => {}
                }
            } else if let Ok(Event::Resize(_, rows)) = event {
                if menu.num_rows != rows {
                    menu.num_rows = rows;
                    menu.redraw()?;
                }
            }
        }

        menu.leave()
    }

    fn write_menu(&mut self) -> io::Result<()> {
        queue!(self.out, cursor::MoveTo(START_X, START_Y))?;
        for (i, line) in self.lines.iter().enumerate().take(self.num_rows as usize) {
            if i == 0 {
                queue!(
                    self.out,
                    Print(line.as_str().on(formats::MENU_SELECTED)),
                    cursor::MoveToNextLine(1)
                )?;
            } else {
                queue!(
                    self.out,
                    Print(" ".to_string() + line),
                    cursor::MoveToNextLine(1)
                )?;
            }
        }
        self.out.flush()
    }

    // TODO make this work with keeping the selected id
    fn redraw(&mut self) -> io::Result<()> {
        execute!(self.out, terminal::Clear(ClearType::All))?;
        self.write_menu()?;
        self.selected_id = 0;
        self.cursor_y = START_Y;
        Ok(())
    }

    fn suspend(&mut self) -> io::Result<()> {
        #[cfg(not(windows))]
        {
            self.leave()?;
            signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP).unwrap();
        }
        Ok(())
    }

    fn resume(&mut self) -> io::Result<()> {
        self.num_rows = Menu::num_rows();
        self.enter()?;
        self.redraw()?;
        Ok(())
    }

    fn move_down(&mut self) -> io::Result<()> {
        self.destyle_at_cursor(self.cursor_y)?;
        self.selected_id += 1;
        self.style_at_cursor(self.cursor_y + 1)?;
        if self.cursor_y + SCROLL_OFFSET != self.num_rows {
            self.cursor_y += 1;
        } else {
            execute!(self.out, terminal::ScrollUp(1))?;
            if (self.selected_id + SCROLL_OFFSET as usize) < self.lines.len() {
                execute!(self.out, cursor::MoveTo(START_X, self.num_rows))?;
                execute!(
                    self.out,
                    Print(
                        self.lines
                            .get(self.selected_id - 1 + SCROLL_OFFSET as usize)
                            .unwrap()
                    )
                )?;
            }
        }
        Ok(())
    }

    fn move_up(&mut self) -> io::Result<()> {
        self.destyle_at_cursor(self.cursor_y)?;
        self.selected_id -= 1;
        self.style_at_cursor(self.cursor_y - 1)?;
        if self.selected_id < SCROLL_OFFSET as usize || self.cursor_y != SCROLL_OFFSET {
            self.cursor_y -= 1;
        } else {
            execute!(self.out, terminal::ScrollDown(1))?;
            if self.selected_id + 1 > SCROLL_OFFSET as usize {
                execute!(self.out, cursor::MoveTo(START_X, START_Y))?;
                execute!(
                    self.out,
                    Print(
                        self.lines
                            .get(self.selected_id - SCROLL_OFFSET as usize)
                            .unwrap()
                    )
                )?;
            }
        }

        Ok(())
    }

    fn style_at_cursor(&mut self, cursor_y: u16) -> io::Result<()> {
        let l: &str = self.lines.get(self.selected_id).unwrap();
        execute!(
            self.out,
            cursor::MoveTo(START_X, cursor_y),
            style::Print(l.on(formats::MENU_SELECTED))
        )
    }

    fn destyle_at_cursor(&mut self, cursor_y: u16) -> io::Result<()> {
        execute!(
            self.out,
            cursor::MoveTo(START_X, cursor_y),
            Print(self.lines.get(self.selected_id).unwrap())
        )
    }

    fn enter(&mut self) -> io::Result<()> {
        execute!(
            self.out,
            style::ResetColor,
            cursor::Hide,
            terminal::EnterAlternateScreen,
            terminal::DisableLineWrap,
        )?;
        terminal::enable_raw_mode()
    }

    fn leave(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        self.out.flush()?;
        execute!(
            io::stderr(),
            style::ResetColor,
            cursor::SetCursorStyle::DefaultUserShape,
            terminal::LeaveAlternateScreen,
            cursor::Show,
        )
    }

    fn exit_and_open(&mut self, selected: Selected) -> io::Result<()> {
        #[cfg(not(windows))]
        {
            let opener = match std::env::var("EDITOR") {
                Ok(val) if !val.is_empty() => val,
                _ => match std::env::consts::OS {
                    "macos" => "open".to_string(),
                    _ => "xdg-open".to_string(),
                },
            };

            let line_num: usize = selected.line;
            let mut command: Command = Command::new(&opener);
            match opener.as_str() {
                "vi" | "vim" | "nvim" | "nano" | "emacs" => {
                    command.arg(format!("+{line_num}"));
                    command.arg(selected.path);
                }
                "hx" => {
                    command.arg(format!("{}:{line_num}", selected.path.display()));
                }
                "code" => {
                    command.arg("--goto");
                    command.arg(format!("{}:{line_num}", selected.path.display()));
                }
                _ => {
                    command.arg(selected.path);
                }
            }
            use std::os::unix::process::CommandExt;
            self.leave()?;

            command.exec();
        }

        #[cfg(windows)]
        {
            Command::new("cmd")
                .arg("/C")
                .arg("start")
                .arg(selected.path)
                .spawn()?;
            menu.leave()?;
        }

        Ok(())
    }
}
