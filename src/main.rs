use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use clap::{App, Arg, SubCommand};
use crate::class_file::constant_pool::ConstantPool;
use crate::class_file::constant_pool::types::Utf8Info;
use crate::class_file::attributing::attribute_parsing::ParsedAttribute;
use crate::vm::code_source::CodeSource;

mod byte_util;
pub mod class_file;
pub mod vm;

#[macro_use]
extern crate bitflags;

fn main() {
    let cli = App::new("Rave Virtual Machine")
        .version("1.0.0")
        .author("TheEpicBlock")
        .about("A java virtual machine implementation in rust")
        .subcommand(
            SubCommand::with_name("parse")
                .about("Parses a .class file and prints it. Similar in function to javap")
                .arg(Arg::with_name("INPUT").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("exec")
                .about("Executes .class file.")
                .arg(Arg::with_name("INPUT").required(true).index(1))
                .arg(Arg::with_name("Classpath").alias("cp").required(false)),
        )
        .get_matches();

    if let Some(subcommand) = cli.subcommand_matches("exec") {
        // let mut class_path = vec![];
        //
        // if let Some(path) = subcommand.value_of("Classpath") {
        //     path.split(":").for_each(|str| {
        //         class_path.push(CodeSource::new(PathBuf::from(str)).unwrap());
        //     })
        // }
        let input_path = subcommand.value_of("INPUT").unwrap();
        let input = Path::new(input_path);

        if !input.exists() {
            println!("Invalid path {}", input_path);
            return;
        }

        let mut reader = BufReader::new(File::open(input).expect("Error reading file"));
        let res = class_file::parse(&mut reader).unwrap();
        let res = class_file::attribute(res).unwrap();
        vm::exec(res);
    }

    if let Some(subcommand) = cli.subcommand_matches("parse") {
        let input_path = subcommand.value_of("INPUT").unwrap();
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
                println!("== Constant Pool ==");
                let mut i = 1;
                for entry in &class.constant_pool {
                    println!("#{}: {:?}", i, entry);
                    i += 1;
                }

                let a_class = class_file::attribute(class).expect("Failed to attribute class");
                println!("== Methods ==");
                for method in a_class.methods {
                    println!("{}", a_class.constant_pool.get_as::<Utf8Info>(method.name_index).unwrap().inner);
                    for attribute in method.attributes {
                        if let ParsedAttribute::Code(code) = attribute {
                            for inst in code.code {
                                println!("{:?}", inst);
                            }
                        }
                    }
                }
            }
            Err(err) => {
                println!("Failed to parse file");
                println!("{:?}", err);
            }
        }
    }
}
