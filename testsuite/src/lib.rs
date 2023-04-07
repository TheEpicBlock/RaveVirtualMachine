use std::io::Cursor;

use vm_core::{ClassLoader, class_loaders::SimpleClassLoader};

// In non-test mode, none of this code will be used because everything was made for test mode
#[cfg_attr(not(test), allow(dead_code))]
// The test names are created from Java classnames
#[allow(non_snake_case)]

fn setup_classloader(bytes: &[u8]) -> impl ClassLoader {
    let classfile = classfile_parser::parse(&mut Cursor::new(bytes)).unwrap();
    SimpleClassLoader::new(classfile)
}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));