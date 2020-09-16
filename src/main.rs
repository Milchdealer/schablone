extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};

mod schablone;
use schablone::build_schablone;

fn main() {
    env_logger::init();
    let matches = App::new("schablone")
                        .version("0.2.1")
                        .author("Milchdealer")
                        .about("Build a schablone from template")
                        .arg(Arg::with_name("name")
                            .help("Name of the schablone")
                            .required(true)
                            .index(1)
                            .takes_value(true))
                        .arg(Arg::with_name("target")
                            .help("Where to write the results")
                            .required(true)
                            .index(2)
                            .takes_value(true))
                        .arg(Arg::with_name("parameters")
                            .short("p")
                            .help("Parameters to render as KEY=VALUE pairs separated by a comma. These take precedence over the 'parameters_file'")
                            .takes_value(true))
                        .arg(Arg::with_name("parameters_file")
                            .short("f")
                            .help("Parameters to render, given the path to a JSON file")
                            .takes_value(true))
                        .arg(Arg::with_name("dry_run")
                            .short("d")
                            .help("Do not actually create the folders and files at the destination. Useful for testing templates and parameters")).get_matches();

    let name = matches.value_of("name").unwrap_or("schablone");
    let target = matches.value_of("target").unwrap_or("schablone_target");
    let parameters = matches.value_of("parameters").unwrap_or("");
    let parameters_file = matches.value_of("parameters_file").unwrap_or("");
    let dry_run = matches.is_present("dry_run");

    build_schablone(name, target, parameters, parameters_file, dry_run);
}
