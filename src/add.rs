use crate::{
    cli,
    error::Result,
    types::{Config, EnvironmentVariable},
};
use clap::{value_t, ArgMatches};
use failure::format_err;

pub fn add(matches: &ArgMatches<'_>) -> Result<()> {
    let config_file = value_t!(matches, "file", String)?;
    let profile = value_t!(matches, "profile", String).ok();
    let name =
        value_t!(matches, "name", String).or_else(|_| cli::input_required("Variable name: "))?;
    let description = value_t!(matches, "description", String)
        .map(|value| if value.is_empty() { None } else { Some(value) })
        .or_else(|_| cli::input_optional("Description: "))?;
    let required = value_t!(matches, "required", bool).unwrap_or_default();

    let mut config = Config::load(&config_file).unwrap_or_default();

    // limit mutable borrow
    {
        // TODO add resolution for user, i.e. how to create a new profile
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