use std::collections::HashSet;
use std::env;
use std::ffi::OsString;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tests");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    // Clear the directory if it isn't empty
    if fs::read_dir(out_dir).unwrap().count() > 0 {
        fs::remove_dir_all(out_dir).unwrap();
        fs::create_dir(out_dir).unwrap();
    }

    let mut tests = vec![];
    let testdir = Path::new("tests");
    let test_names: HashSet<_> = fs::read_dir(testdir).unwrap()
        .map(|file| file.unwrap().path().file_stem().unwrap().to_owned())
        .collect();

    for test_name in test_names {
        let java_file = testdir.join(&test_name).with_extension("java");
        let rust_file = testdir.join(&test_name).with_extension("rs");

        let out_dir = out_dir.join(&test_name);
        println!("{}", out_dir.display());
        fs::create_dir(&out_dir).unwrap();
        Command::new("javac")
            .arg(java_file)
            .arg("-d").arg(&out_dir)
            .status().unwrap();

        tests.push(Test {
            name: test_name,
            out_dir: out_dir,
            rust_file: rust_file,
        })
    }

    let mut output = String::new();

    for test in tests {
        let test_name = test.name.to_str().unwrap();
        let rust_file = test.rust_file.canonicalize().unwrap();
        let rust_file = rust_file.to_str().unwrap();

        let output_files: Vec<_> = fs::read_dir(test.out_dir).unwrap().map(|f| f.unwrap()).collect();
        assert_eq!(output_files.len(), 1);
        let java_file = &output_files[0];
        let java_file = java_file.path().canonicalize().unwrap();
        let java_file = java_file.to_str().unwrap();

        write!(output, r#"
            #[cfg(test)]
            mod {test_name} {{
                #[test]
                fn test() {{
                    // setup_classloader comes from lib.rs
                    run(crate::setup_classloader(include_bytes!("{java_file}")));
                }}

                include!("{rust_file}");
            }}
        "#).unwrap();
    }

    fs::write(out_dir.join("generated.rs"), output).unwrap();
}

struct Test {
    name: OsString,
    rust_file: PathBuf,
    out_dir: PathBuf,
}