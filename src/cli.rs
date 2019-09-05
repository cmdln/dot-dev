use crate::error::Result;
use crossterm::TerminalInput;
use failure::format_err;
use std::io::{self, prelude::*, Stdout};

pub(crate) fn input_optional<S: AsRef<str>>(prompt: S) -> Result<Option<String>> {
    prompt_input(prompt.as_ref(), &mut io::stdout(), &crossterm::input()).and_then(|value| {
        if value.is_empty() {
            Ok(None)
        } else {
            value
                .parse()
                .map_err(failure::Error::from)
                .map(Option::Some)
        }
    })
}

pub(crate) fn input_required<S: AsRef<str>>(prompt: S) -> Result<String> {
    let mut interaction = Interaction::new(prompt);
    interaction
        .find(|result| {
            result
                .as_ref()
                .map(|value| !value.is_empty())
                .unwrap_or_default()
        })
        .ok_or_else(|| format_err!("Interrupted!"))?
}

struct Interaction {
    prompt: String,
    stdout: Stdout,
    input: TerminalInput,
}

impl Interaction {
    fn new<S: AsRef<str>>(prompt: S) -> Self {
        Self {
            prompt: prompt.as_ref().to_owned(),
            stdout: io::stdout(),
            input: crossterm::input(),
        }
    }
}

impl Iterator for Interaction {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(prompt_input(&self.prompt, &mut self.stdout, &self.input))
    }
}

fn prompt_input(prompt: &str, stdout: &mut Stdout, input: &TerminalInput) -> Result<String> {
    print!("{}", prompt);
    stdout
        .flush()
        .map_err(failure::Error::from)
        .and_then(|_| input.read_line().map_err(failure::Error::from))
}
