///! Module implementing the various profile related commands.
use crate::{
    cli,
    error::Result,
    types::{Config, Profile},
};
use clap::{value_t, App, Arg, ArgMatches, SubCommand};

/// Execute the profile subcommand which doesn't do anything itself, rather it
/// expects one addition subcommand such as add, list or remove.
pub(crate) fn exec(matches: &ArgMatches<'_>) -> Result<()> {
    match &matches.subcommand {
        Some(subcommand) if subcommand.name == "add" => add(&subcommand.matches),
        Some(subcommand) if subcommand.name == "list" => list(&subcommand.matches),
        Some(subcommand) => {
            println!("{}", subcommand.name);
            println!("{}", subcommand.matches.usage());
            Ok(())
        }
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

pub(crate) fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("profile")
        .about("Commands for working with profiles. A profile is a named collection of environment variables.")
        .subcommand(
            SubCommand::with_name("add")
                .arg(
                    Arg::with_name("name")
                        .help("Name of the profile to add.")
                        .short("n")
                        .long("name")
                        .takes_value(true)
                )
                .arg(crate::define_file_arg())
        )
        .subcommand(SubCommand::with_name("list").arg(crate::define_file_arg()))
}

fn add(matches: &ArgMatches<'_>) -> Result<()> {
    let config_file = value_t!(matches, "file", String)?;
    let name =
        value_t!(matches, "name", String).or_else(|_| cli::text_required("Profile name: "))?;
    let config = Config::load(&config_file)?;
    let mut profiles = config.profiles;
    if profiles.iter().any(|profile| profile.name == name) {
        println!("Profile, \"{}\", already exists.", name);
        Ok(())
    } else {
        profiles.push(Profile {
            name,
            ..Profile::default()
        });
        let config = Config { profiles, ..config };
        config.save(&config_file)
    }
}

fn list(matches: &ArgMatches<'_>) -> Result<()> {
    let config_file = value_t!(matches, "file", String)?;
    let config = Config::load(&config_file)?;
    let mut names = config
        .profiles
        .iter()
        .map(|profile| profile.name.as_str())
        .collect::<Vec<&str>>();
    names.insert(0, "default");
    let names = &names[..];
    println!("Profiles: {}", names.join(", "));
    Ok(())
}
