mod add;
mod cli;
mod error;
mod types;

use clap::{App, Arg, SubCommand};
use error::Result;
use std::{
    io::{Error, ErrorKind},
    process,
};

fn main() -> Result<()> {
    let app = App::new("dot-dev").subcommand(define_add());
    let matches = app.get_matches();
    if let Some(matches) = matches.subcommand_matches("add") {
        match add::add(&matches) {
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
    } else {
        println!("{}", matches.usage());
        std::process::exit(1);
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
