use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use clap::{App, Arg, SubCommand};

pub mod class_file;
mod byte_util;

fn main() {
    let cli = App::new("Rave Virtual Machine")
        .version("1.0.0")
        .author("TheEpicBlock")
        .about("A java virtual machine implementation in rust")
        .subcommand(SubCommand::with_name("parse")
            .about("Parses a .class file and prints it. Similar in function to javap")
            .arg(Arg::with_name("INPUT").required(true).index(1)))
        .get_matches();

    if let Some(subcommand) = cli.subcommand_matches("parse") {
        let input_path= subcommand.value_of("INPUT").unwrap();
        let input = Path::new(input_path);

        if !input.exists() {
            println!("Invalid path {}", input_path);
            return;
        }

        let mut reader = BufReader::new(File::open(input).expect("Error reading file"));
        let res = class_file::parse(&mut reader);
        match res {
            Ok(class) => {
                println!("Successfully parsed file");
                let mut i = 0;
                for entry in class.constant_pool {
                    println!("#{}: {:?}", i, entry);
                    i += 1;
                }
            }
            Err(err) => {
                println!("Failed to parse file");
                println!("{}", err);
            }
        }
    }
}