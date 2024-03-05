#![feature(exit_status_error)]

use std::collections::HashSet;
use std::env;
use std::ffi::OsString;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use walkdir::WalkDir;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=jasm/src");
    println!("cargo:rerun-if-changed=jasm/build.gradle.kts");
    println!("cargo:rerun-if-changed=jasm/gradle.properties");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    // Clear the directory if it isn't empty
    if fs::read_dir(out_dir).unwrap().count() > 0 {
        fs::remove_dir_all(out_dir).unwrap();
        fs::create_dir(out_dir).unwrap();
    }

    let testdir = Path::new("src").canonicalize().expect("Need to be able to canonicalize src dir");
    println!("{}", testdir.display());

    for entry in WalkDir::new(&testdir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            continue;
        }

        let create_target_dir = || {
            let target = out_dir.join(entry.path().canonicalize().unwrap().parent().unwrap().strip_prefix(&testdir).unwrap());
            fs::create_dir_all(&target).expect("Couldn't mkdir for output files");
            target
        };

        
        if entry.path().extension().is_some_and(|p| p == "jasm") {
            let target = create_target_dir();
            println!("Compiling jasm file from {} into {}", entry.path().display(), target.display());
            run_jasm(entry.path(), &target);
        } else if entry.path().extension().is_some_and(|p| p == "java") {
            let target = create_target_dir();
            println!("Compiling java file from {} into {}", entry.path().display(), target.display());
            Command::new("javac")
                .arg(entry.path())
                .arg("-d").arg(&target)
                .status().unwrap().exit_ok().unwrap();
        }
    }

    // for entry in WalkDir::new(testdir).into_iter().filter_entry(|f| f.path().extension().is_some_and(|p| p == "java")).filter_map(|e| e.ok()) {
    //     let target = out_dir.join(entry.path().canonicalize().unwrap().strip_prefix(testdir).unwrap());
    //     Command::new("javac")
    //         .arg(entry.path())
    //         .arg("-d").arg(&target)
    //         .status().unwrap().exit_ok().unwrap();
    // }
}

fn run_jasm(input: &Path, output_dir: &Path) {
    let jasm = Path::new("jasm");

    #[cfg(not(target_os = "windows"))]
    let gradlew = jasm.join("gradlew");
    #[cfg(target_os = "windows")]
    let gradlew = jasm.join("gradlew.bat");

    let output_dir = output_dir.canonicalize().unwrap();
    let input = input.canonicalize().unwrap();

    println!("--args=\"-o '{}' -i '{}' '{}'\"", output_dir.display(), input.parent().unwrap().display(), input.file_name().unwrap().to_str().unwrap());

    Command::new(gradlew.canonicalize().unwrap())
        .current_dir(jasm)
        .arg("run")
        .arg(format!("--args=-o '{}' -i '{}' '{}'", output_dir.display(), input.parent().unwrap().display(), input.file_name().unwrap().to_str().unwrap()))
        .status().unwrap().exit_ok().unwrap();
}