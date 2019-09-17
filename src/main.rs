mod add;
mod cli;
mod error;
mod types;

use clap::{App, Arg, ArgMatches, SubCommand};
use error::Result;
use std::{
    io::{Error, ErrorKind},
    process,
};

fn main() -> Result<()> {
    let app = App::new("dot-dev")
        .arg(Arg::with_name("quiet").short("q"))
        .arg(Arg::with_name("verbosity").short("v"))
        .subcommand(define_add());
    let matches = app.get_matches();
    stderrlog::new()
        .module(module_path!())
        .quiet(matches.is_present("quiet"))
        .verbosity(
            matches.occurrences_of("verbosity") as usize
                + if matches.is_present("quiet") { 0 } else { 2 },
        )
        .init()?;
    if let Some(matches) = matches.subcommand_matches("add") {
        handle_interrupt(add::add, &matches)
    } else {
        println!("{}", matches.usage());
        std::process::exit(1);
    }
}

// Some cli functions halt the process as expected, some require return a custom IO error of kind
// Interrupted that then need to be downcast and matched, otherwise the main function prints the
// debug display of the error which isn't a great user experience
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

fn define_add<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("add")
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("description")
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
            Arg::with_name("file")
                .short("f")
                .long("file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .takes_value(true),
        )
}
