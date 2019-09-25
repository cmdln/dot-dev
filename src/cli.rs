use crate::error::Result;
use crossterm::{ClearType, InputEvent, KeyEvent, RawScreen, SyncReader, Terminal, TerminalCursor};
use failure::format_err;
use std::{
    fmt::Display,
    io::{self},
};

pub(crate) fn text<S: AsRef<str>>(prompt: S) -> Result<Option<String>> {
    prompt_for_text(prompt)
        .map(|result| result.map(|value| if value.is_empty() { None } else { Some(value) }))
        .next()
        .ok_or_else(|| format_err!("Interrupted"))?
}

pub(crate) fn text_required<S: AsRef<str>>(prompt: S) -> Result<String> {
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

pub(crate) fn answer_yes_no<S: AsRef<str>>(prompt: S) -> Result<bool> {
    choose(prompt, 0, vec![Answer::Yes, Answer::No]).map(|answer| answer.as_bool())
}

fn choose<V: Display + PartialEq, S: AsRef<str>>(
    prompt: S,
    selected: usize,
    choices: Vec<V>,
) -> Result<V> {
    let mut choices = Choices::try_new(prompt.as_ref().to_owned(), selected, choices)?;
    let choice = choices.find(|result| {
        result.is_err()
            || result
                .as_ref()
                .map(|result| result.is_select())
                .unwrap_or_default()
    });
    match choice {
        Some(Ok(Choice::Select(choice))) => Ok(choice),
        Some(Ok(_)) => Err(format_err!("Incorrectly found a result without a value!")),
        Some(Err(error)) => Err(error),
        None => Err(format_err!("Failed to find any result!")),
    }
}

fn prompt_for_text<S: AsRef<str>>(prompt: S) -> impl Iterator<Item = Result<String>> {
    std::iter::repeat_with(|| (crossterm::terminal(), crossterm::input())).map(
        move |(terminal, input)| {
            terminal.write(prompt.as_ref())?;
            input.read_line().map_err(failure::Error::from)
        },
    )
}

#[derive(PartialEq)]
enum Answer {
    Yes,
    No,
}

impl Display for Answer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(match self {
            Self::Yes => "yes",
            Self::No => "no",
        })
    }
}

impl Answer {
    fn as_bool(&self) -> bool {
        match self {
            Self::Yes => true,
            Self::No => false,
        }
    }
}

struct UI {
    prompt: String,
    terminal: Terminal,
    cursor: TerminalCursor,
    reader: SyncReader,
}

impl UI {
    fn new(prompt: String) -> Self {
        let terminal = crossterm::terminal();
        let cursor = crossterm::cursor();
        let input = crossterm::input();
        let reader = input.read_sync();
        Self {
            prompt,
            terminal,
            cursor,
            reader,
        }
    }

    fn prompt_read_next<D: Display>(&mut self, value: &D) -> Result<Option<InputEvent>> {
        let _raw = RawScreen::into_raw_mode()?;
        self.terminal.clear(ClearType::CurrentLine)?;
        self.cursor.restore_position()?;
        self.terminal.write(format!("{}{}", self.prompt, value))?;
        let event = self.reader.next();
        RawScreen::disable_raw_mode()?;
        Ok(event)
    }
}

struct Choices<T: Display + PartialEq> {
    ui: UI,
    has_next: bool,
    selected: usize,
    values: Vec<T>,
}

impl<T: Display + PartialEq> Choices<T> {
    fn try_new(prompt: String, selected: usize, values: Vec<T>) -> Result<Self> {
        let ui = UI::new(prompt);
        ui.cursor.save_position()?;
        let has_next = true;
        Ok(Self {
            ui,
            has_next,
            selected,
            values,
        })
    }

    fn handle(&mut self) -> Result<Choice<T>> {
        match self.ui.prompt_read_next(&self.values[self.selected])? {
            Some(InputEvent::Keyboard(KeyEvent::Char('\n')))
            | Some(InputEvent::Keyboard(KeyEvent::Enter)) => {
                self.has_next = false;
                println!();
                self.select()
            }
            Some(InputEvent::Keyboard(KeyEvent::Ctrl('c'))) => {
                self.has_next = false;
                Err(io::Error::new(io::ErrorKind::Interrupted, "keyboard interrupt").into())
            }
            Some(InputEvent::Keyboard(KeyEvent::Tab))
            | Some(InputEvent::Keyboard(KeyEvent::Down))
            | Some(InputEvent::Keyboard(KeyEvent::Right)) => {
                self.selected = self.increment();
                Ok(Choice::Change)
            }
            Some(InputEvent::Keyboard(KeyEvent::BackTab))
            | Some(InputEvent::Keyboard(KeyEvent::Up))
            | Some(InputEvent::Keyboard(KeyEvent::Left)) => {
                self.selected = self.decrement();
                Ok(Choice::Change)
            }
            _ => Ok(Choice::NoOp),
        }
    }

    fn select(&mut self) -> Result<Choice<T>> {
        Ok(Choice::Select(self.values.swap_remove(self.selected)))
    }

    fn increment(&self) -> usize {
        if self.selected + 1 >= self.values.len() {
            0
        } else {
            self.selected + 1
        }
    }

    fn decrement(&self) -> usize {
        if self.selected == 0 {
            self.values.len() - 1
        } else {
            self.selected - 1
        }
    }
}

impl<T: Display + PartialEq> Iterator for Choices<T> {
    type Item = Result<Choice<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_next {
            Some(self.handle())
        } else {
            None
        }
    }
}

enum Choice<T> {
    Change,
    NoOp,
    Select(T),
}

impl<T> Choice<T> {
    fn is_select(&self) -> bool {
        match self {
            Self::Select(_) => true,
            _ => false,
        }
    }
}
