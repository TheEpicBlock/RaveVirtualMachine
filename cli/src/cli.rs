use clap::{Parser, ValueHint};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "Rave Virtual Machine", version = "0.1.0", author = "TheEpicBlock <git.teb@theepicblock.nl>")]
pub enum RaveCliFormat {
    Parse(Parse),
    Run(Run),
}

#[derive(Parser)]
pub struct Parse {
    #[clap(parse(from_os_str), value_hint = ValueHint::FilePath)]
    pub input: PathBuf
}

#[derive(Parser)]
pub struct Run {
    #[clap(parse(from_os_str), value_hint = ValueHint::FilePath)]
    pub file: PathBuf
}