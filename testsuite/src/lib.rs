use std::io::Cursor;

use vm_core::{ClassLoader, class_loaders::SimpleClassLoader};

fn setup_classloader(bytes: &[u8]) -> impl ClassLoader {
    let classfile = classfile_parser::parse(&mut Cursor::new(bytes)).unwrap();
    SimpleClassLoader::new(classfile)
}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));