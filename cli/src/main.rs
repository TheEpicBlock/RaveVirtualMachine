mod cli;

use crate::cli::RaveCliFormat;
use clap::Parser;
use std::io::BufReader;
use std::fs::File;
use classfile_parser::constant_pool::ConstantPool;
use classfile_parser::constant_pool::types::Utf8Info;
use classfile_parser::attributes::AttributeEntry;
use std::error::Error;
use std::fmt::Display;

fn main() {
    let cli = RaveCliFormat::parse();

    match cli {
        RaveCliFormat::Parse(parse) => {
            let input = parse.input;

            if !input.exists() {
                println!("Invalid path {}", input.display());
                return;
            }

            let mut reader = BufReader::new(File::open(input).expect("Error reading file"));
            let res = classfile_parser::parse(&mut reader);

            match res {
                Ok(class) => {
                    println!("Successfully parsed file");
                    println!("== Constant Pool ==");
                    let mut i = 1;
                    for entry in &class.constant_pool {
                        println!("#{}: {:?}", i, entry);
                        i += 1;
                    }

                    println!("== Methods ==");
                    for method in class.methods {
                        println!("{}", class.constant_pool.get_as::<Utf8Info>(method.name_index).unwrap().inner);
                        for attribute in method.attributes {
                            if let AttributeEntry::Code(code) = attribute {
                                for inst in code.code {
                                    println!("{:?}", inst);
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("Failed to parse file. Caused by:");
                    print_err(err);
                }
            }
        }
    }
}

fn print_err(err: impl Error + Display) {
    match err.source() {
        Some(source) => {
            print_err(source);
        }
        None => {}
    }
    println!(" - {}", err);
}

