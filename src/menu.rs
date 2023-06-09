/*
user selects a line from a file with j,k and then
enter key or by pressing the letters/numbers on the
left side of the match, open with $EDITOR
*/

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

use crate::formats::{self, MENU_SELECTED};
use crate::printer;
use crate::searcher::DirPointer;
pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, queue,
    style::{self, Attribute, Color, Print, Stylize},
    terminal::{self, ClearType},
    Command, ErrorKind,
};
use std::io::{self, Write};
use std::rc::Rc;

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
    draw_loop(out, lines)?;

    Ok(())
}

fn draw_loop<W>(out: &mut W, lines: Vec<String>) -> io::Result<()>
where
    W: Write,
{
    // this will cause errors if there are more than 2^(16)-1=65535 matches
    let mut selected: u16 = 0;
    let max_selected_id = lines.len() as u16 - 1;
    let size: Option<(u16, u16)> = terminal::size().ok();
    loop {
        if size.is_some() && size != terminal::size().ok() {
            queue!(out, terminal::Clear(ClearType::All),)?;
        }

        for (i, line) in lines.iter().enumerate() {
            // this only works until there is a style reset
            queue!(out, cursor::MoveTo(1, i as u16 + 1))?;
            if i as u16 == selected {
                queue!(out, MENU_SELECTED)?;
            }
            queue!(out, Print(line))?;
        }

        out.flush()?;

        // read_char loops till there is a key press
        match read_char() {
            Some('j') => {
                if selected < max_selected_id - 1 {
                    selected += 1;
                }
            }
            Some('k') => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            Some('q') | None => break,
            Some(_) => {}
        };
    }

    // TODO make a leave function so that
    // when an error occurs above still call
    // these cleanup functions
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

pub fn read_char() -> Option<char> {
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            modifiers,
            state: _,
        })) = event::read()
        {
            if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) && c == 'c' {
                return None;
            } else {
                return Some(c);
            }
        }
    }
}
