use std::{env, process::Command};

include!("src/config.rs");

pub fn compile_resources(source_dirs: &[&str], gresource: &str, target: &str) {
    let out_dir = env::var("OUT_DIR").unwrap();

    let mut command = Command::new("glib-compile-resources");
    command
        .arg("--target")
        .arg(format!("{}/{}", out_dir, target))
        .arg(gresource);

    for dir in source_dirs {
        command.arg("--sourcedir").arg(dir);
    }

    let status = command.status().unwrap();

    assert!(
        status.success(),
        "glib-compile-resources failed with exit status {}",
        status
    );

    println!("cargo:rerun-if-changed={}", gresource);
    let mut command = Command::new("glib-compile-resources");

    for dir in source_dirs {
        command.arg("--sourcedir").arg(dir);
    }

    let output = command
        .arg("--generate-dependencies")
        .arg(gresource)
        .output()
        .unwrap()
        .stdout;
    let output = String::from_utf8(output).unwrap();
    for dep in output.split_whitespace() {
        if dep.contains(".ui") {
            continue;
        }
        println!("cargo:rerun-if-changed={}", dep);
    }
}

pub fn compile_blueprint(source_dir: &str, target: &str) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut command = Command::new(BLUEPRINT_COMPILER_PATH);
    command
        .arg("batch-compile")
        .arg(format!("{}/{}", out_dir, target))
        .arg(source_dir);

    for path in glob::glob(&format!("{}/ui/**/*.blp", source_dir)).unwrap() {
        let p = path.unwrap();
        println!("cargo:rerun-if-changed={}", p.display());
        command.arg(p);
    }

    let status = command.status().unwrap();

    if !status.success() {
        panic!("blueprint-compiler failed with exit status {}", status);
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    compile_blueprint(&format!("{}/{}", manifest_dir, "data"), "data");
    compile_resources(
        &[
            &format!("{}/{}", out_dir, "data"),
            &format!("{}/{}", manifest_dir, "data"),
        ],
        "data/dev.Cogitri.Health.gresource.xml",
        "compiled.gresource",
    );
}
