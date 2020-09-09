extern crate clap;
use clap::{Arg, App, SubCommand};

mod schablone;
use schablone::{new_schablone, build_schablone};

fn main() {
    println!("Schablone");
    let matches = App::new("schablone")
                        .version("0.0.1")
                        .author("Milchdealer")
                        .about("schablone.")
                        .subcommand(SubCommand::with_name("new")
                            .about("Create a new schablone")
                            .arg(Arg::with_name("name")
                                .help("Name of the new schablone")
                                .required(true)
                                .index(1)
                                .takes_value(true)))
                        .subcommand(SubCommand::with_name("build")
                            .about("Build from template")
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
                                .help("Parameters to render, either KEY=VALUE pairs separated by a comma or a path to a JSON")
                                .takes_value(true))).get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        let name = matches.value_of("name").unwrap_or("schablone");
        new_schablone(name);
    } else if let Some(matches) = matches.subcommand_matches("build") {
        let name = matches.value_of("name").unwrap_or("schablone");
        let target = matches.value_of("target").unwrap_or("schablone_target");
        let parameters = matches.value_of("parameters").unwrap_or("");

        build_schablone(name, target, parameters);
    }
}
