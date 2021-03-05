use std::collections::HashMap;

#[macro_use]
extern crate clap;

use clap::{App, Arg, SubCommand};

use crossword;

fn main() {
    crossword::logging::init_logger(true);

    let setting_names = ["num-per-gen", "num-children", "max-rounds", "seed", "moves-between-scores"];
    let mut setting_args: Vec<Arg> = vec![];
    for setting_name in setting_names.iter() {
        let full_name = format!("--{}", setting_name);
        setting_args.push(Arg::with_name(&setting_name).long(&setting_name).takes_value(true));
    }

    let matches = App::new("Crossword pedigree")
        .version("1.0")
        .author("Kath Nicholls")
        .about("Generates a crossword from a set of clues and answers")
        .arg(Arg::with_name("CLUE_FILE")
                .required(true))
        .args(&setting_args)
        .get_matches();

    let mut settings_map: HashMap<&str, usize> = HashMap::new();
    for setting_name in setting_names.iter() {
        let setting_value = value_t!(matches, *setting_name, usize);
        match setting_value {
            Ok(value) => { settings_map.insert(setting_name, value); },
            Err(error) if error.kind == clap::ErrorKind::ArgumentNotFound => (),
            Err(error) => { panic!("Failed to parse arguments - invalid argument given. {}", error); },
        }
    }
    println!("{:?}", settings_map);

    let mut generator = crossword::generator::CrosswordGenerator::new_from_file(matches.value_of("CLUE_FILE").unwrap(),
    settings_map);

}
