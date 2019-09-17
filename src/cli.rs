use crate::error::Result;
use crossterm::{ClearType, InputEvent, KeyEvent, RawScreen};
use failure::format_err;
use std::io::{self, prelude::*};

pub(crate) fn input_optional<S: AsRef<str>>(prompt: S) -> Result<Option<String>> {
    prompt_for_text(prompt)
        .map(|result| result.map(|value| if value.is_empty() { None } else { Some(value) }))
        .next()
        .ok_or_else(|| format_err!("Interrupted"))?
}

pub(crate) fn input_required<S: AsRef<str>>(prompt: S) -> Result<String> {
    prompt_for_text(prompt)
        .skip_while(|value| {
            value
                .as_ref()
                .map(|value| {
                    if value.is_empty() {
                        println!("A response is required!");
                    }
                    value.is_empty()
                })
                .unwrap_or_default()
        })
        .next()
        .ok_or_else(|| format_err!("Interrupted"))?
}

fn prompt_for_text<S: AsRef<str>>(prompt: S) -> impl Iterator<Item = Result<String>> {
    std::iter::repeat_with(|| (io::stdout(), crossterm::input())).map(move |(mut stdout, input)| {
        print!("{}", prompt.as_ref());
        stdout.flush()?;
        input.read_line().map_err(failure::Error::from)
    })
}

pub(crate) fn prompt<S: AsRef<str>>(prompt: S) -> Result<bool> {
    let _raw = RawScreen::into_raw_mode()?;
    let terminal = crossterm::terminal();
    let cursor = crossterm::cursor();
    cursor.save_position()?;
    let input = crossterm::input();
    let mut reader = input.read_sync();
    let mut value = false;
    loop {
        terminal.clear(ClearType::CurrentLine)?;
        cursor.reset_position()?;
        print!(
            "{}{}",
            prompt.as_ref(),
            if value { "true" } else { "false" }
        );
        io::stdout().flush()?;
        let event = reader.next();
        match event {
            Some(InputEvent::Keyboard(KeyEvent::Char('y'))) => value = true,
            Some(InputEvent::Keyboard(KeyEvent::Char('n'))) => value = false,
            Some(InputEvent::Keyboard(KeyEvent::Char('\n'))) => break,
            Some(InputEvent::Keyboard(KeyEvent::Ctrl('c'))) => {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "keyboard interrupt").into())
            }
            _ => {}
        }
    }
    RawScreen::disable_raw_mode()?;
    Ok(value)
}
