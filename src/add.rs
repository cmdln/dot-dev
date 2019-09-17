use crate::{
    cli,
    error::Result,
    types::{Config, EnvironmentVariable},
};
use clap::{value_t, ArgMatches};
use failure::format_err;
use log::debug;

pub fn add(matches: &ArgMatches<'_>) -> Result<()> {
    let config_file = value_t!(matches, "file", String)?;
    let profile_arg = value_t!(matches, "profile", String).ok();
    let name =
        value_t!(matches, "name", String).or_else(|_| cli::input_required("Variable name: "))?;
    let description = value_t!(matches, "description", String)
        .map(|value| if value.is_empty() { None } else { Some(value) })
        .or_else(|_| cli::input_optional("Description: "))?;
    let required = value_t!(matches, "required", bool).unwrap_or_default();

    let config = Config::load(&config_file).unwrap_or_default();

    // TODO add resolution for user, i.e. how to create a new profile
    let profile = config.profile(&profile_arg).ok_or_else(|| {
        format_err!(
            "Could not find {}",
            profile_arg
                .as_ref()
                .map(|profile| format!("a profile named {}", profile))
                .unwrap_or_else(|| String::from("default profile"))
        )
    })?;

    let var = EnvironmentVariable {
        name,
        description,
        required,
    };

    debug!("Original profile {:?}", profile);
    let profile = profile.to_owned().add(var)?;
    debug!("Profile after adding {:?}", profile);

    let config = if profile_arg.is_some() {
        config.upsert_profile(profile)
    } else {
        config.update_default_profile(profile)
    };
    debug!("Saving {}", config_file);
    config.save(&config_file)
}
