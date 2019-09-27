use crate::{
    cli,
    error::Result,
    types::{Config, EnvironmentVariable},
};
use clap::{value_t, App, Arg, ArgMatches, SubCommand};
use failure::format_err;
use log::debug;

pub(crate) fn exec(matches: &ArgMatches<'_>) -> Result<()> {
    let config_file = value_t!(matches, "file", String)?;
    let profile_arg = value_t!(matches, "profile", String).ok();
    let name =
        value_t!(matches, "name", String).or_else(|_| cli::text_required("Variable name: "))?;
    let description = value_t!(matches, "description", String)
        .map(|value| if value.is_empty() { None } else { Some(value) })
        .or_else(|_| cli::text("Description: "))?;
    let default_value = value_t!(matches, "default", String)
        .map(|value| if value.is_empty() { None } else { Some(value) })
        .or_else(|_| cli::text("Default value: "))?;
    let required = value_t!(matches, "required", bool).unwrap_or_default();

    let config = Config::load(&config_file).unwrap_or_default();

    let profile = config.profile(&profile_arg).ok_or_else(|| {
        format_err!(
            "Could not find profile, \"{0}\". You may need to add it with \"dot-dev profile add {0} -f {1}\".",
            profile_arg
                .as_ref()
                .map(|profile| format!("a profile named {}", profile))
                .unwrap_or_else(|| String::from("default profile")),
                config_file
        )
    })?;

    let var = EnvironmentVariable {
        name,
        description,
        required,
        default_value,
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

pub(crate) fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("add")
        .about("Add a new environment variable")
        .arg(
            Arg::with_name("name")
                .help("Name for new environment variable, usually all upper case.")
                .short("n")
                .long("name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("description")
                .help("Optional description.")
                .short("d")
                .long("description")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("required")
                .short("r")
                .long("required")
                .conflicts_with("optional")
                .default_value("false"),
        )
        .arg(
            Arg::with_name("optional")
                .short("o")
                .long("optional")
                .conflicts_with("required"),
        )
        .arg(
            Arg::with_name("default")
                .help(
                    "What default value to present for this variable when generating the dot file.",
                )
                .short("D")
                .long("default-value")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .takes_value(true),
        )
        .arg(crate::define_file_arg())
}
