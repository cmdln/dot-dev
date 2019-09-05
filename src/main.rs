extern crate clap;
extern crate dot_dev;

use clap::{App, Arg, SubCommand};
use dot_dev::Result;

fn main() -> Result<()> {
    let app = App::new("dot-dev").subcommand(define_add());
    let matches = app.get_matches();
    if let Some(matches) = matches.subcommand_matches("add") {
        dot_dev::add(&matches)
    } else {
        println!("{}", matches.usage());
        std::process::exit(1);
    }
}

fn define_add<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("add")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .required(true)
                .takes_value(true),
        ).arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .takes_value(true),
        )
}