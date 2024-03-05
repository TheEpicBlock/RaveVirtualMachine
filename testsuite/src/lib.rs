use std::io::Cursor;

use vm_core::{ClassLoader, class_loaders::SimpleClassLoader};

// In non-test mode, none of this code will be used because everything was made for test mode
#[cfg_attr(not(test), allow(dead_code))]

/// Tests which test very basic jvm functionality.
/// 
/// All tests are single classes which define a `run` function returning an int.
/// This int is compared with the expected output
mod basic;

fn setup_classloader(bytes: &[u8]) -> impl ClassLoader {
    let classfile = classfile_parser::parse(&mut Cursor::new(bytes)).unwrap();
    SimpleClassLoader::new(classfile)
}

macro_rules! include_class {
    ($str:literal) => {
        include_bytes!(concat!(env!("OUT_DIR"), $str))
    };
}
pub(crate) use include_class;