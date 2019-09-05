extern crate clap;
extern crate crossterm;
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod error;
mod types;

use clap::{value_t, ArgMatches};
use failure::format_err;

use std::io::{self, prelude::*};

pub use error::*;
pub use types::*;

pub fn add(matches: &ArgMatches) -> Result<()> {
    let config_file = value_t!(matches, "file", String)?;
    let profile = value_t!(matches, "profile", String).ok();
    let name = value_t!(matches, "name", String).or_else(|_| prompt_name())?;
    let description = Some(value_t!(matches, "name", String).unwrap_or_else(|_| String::from("")));
    let required = value_t!(matches, "required", bool).unwrap_or_default();

    let mut config = Config::load(&config_file).unwrap_or_default();

    // limit mutable borrow
    {
        // TODO add resolution for user
        let profile = config.profile_mut(&profile).ok_or_else(|| {
            format_err!(
                "Could not find {}",
                profile
                    .map(|profile| format!("a profile named {}", profile))
                    .unwrap_or_else(|| String::from("default profile"))
            )
        })?;

        let var = EnvironmentVariable {
            name,
            description,
            required,
        };
        profile.add(var)?;
    }

    Config::save(&config, &config_file)
}

fn prompt_name() -> Result<String> {
    // TODO loop until not empty response
    let input = crossterm::input();
    print!("Variable name: ");
    io::stdout().flush()?;
    input.read_line().map_err(|error| error.into())
}
