use std::path::{Path, PathBuf};
use crate::vm::code_source::CodeSource::DIR;

pub enum CodeSource {
    //TODO JAR,
    DIR(PathBuf)
}

impl CodeSource {
    pub(crate) fn new(path: PathBuf) -> Result<Self, ()> {
        if !path.exists() {
            return Err(());
        }
        if path.is_dir() {
            return Ok(DIR(path));
        }
        return Err(());
    }
}