mod add;
mod cli;
mod error;
mod profile;
mod types;

use clap::{App, Arg, ArgMatches};
use error::Result;
use std::{
    io::{Error, ErrorKind},
    process,
};

fn main() -> Result<()> {
    let app = App::new("dot-dev")
        .arg(Arg::with_name("quiet").short("q"))
        .arg(Arg::with_name("verbosity").short("v"))
        .subcommand(add::subcommand())
        .subcommand(profile::subcommand());
    let matches = app.get_matches();
    stderrlog::new()
        .module(module_path!())
        .quiet(matches.is_present("quiet"))
        .verbosity(
            matches.occurrences_of("verbosity") as usize
                + if matches.is_present("quiet") { 0 } else { 2 },
        )
        .init()?;
    match &matches.subcommand {
        Some(subcommand) if subcommand.name == "add" => {
            handle_interrupt(add::exec, &subcommand.matches)
        }
        Some(subcommand) if subcommand.name == "profile" => {
            handle_interrupt(profile::exec, &subcommand.matches)
        }
        _ => {
            println!("{}", matches.usage());
            std::process::exit(1);
        }
    }
}

/// Some cli functions halt the process as expected, some require returning a custom IO error of
/// kind Interrupted that thens need to be downcast and matched, otherwise the main function prints
/// the debug display of the error which isn't a great user experience
fn handle_interrupt(
    cmd: impl Fn(&ArgMatches<'_>) -> Result<()>,
    matches: &ArgMatches<'_>,
) -> Result<()> {
    match cmd(&matches) {
        Ok(_) => Ok(()),
        Err(error) => match error.downcast::<Error>() {
            Ok(io_error) => match io_error.kind() {
                ErrorKind::Interrupted => {
                    println!("^C");
                    process::exit(1);
                }
                _ => Err(io_error.into()),
            },
            Err(error) => Err(error),
        },
    }
}

/// Define a file argument for any subcommands.
fn define_file_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("file")
        .help("JSON file to save to and/or load from.")
        .short("f")
        .long("file")
        .required(true)
        .takes_value(true)
}
