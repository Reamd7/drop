use flate2::write::GzEncoder;
use flate2::Compression;
use glob::glob;
use std::env;
use std::fs::File;
use std::process::Command;

fn main() {
    let cargo_profile = env::var("PROFILE").unwrap();
    let webpack_mode = if cargo_profile == "release" {
        "production"
    } else {
        "development"
    };
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build/libquickjs.a");

    println!("cargo:rustc-link-search=native=build");
    println!("cargo:rustc-link-lib=quickjs");

    // execute webpack to build src/modules_js/**/*.ts files
    Command::new("npx")
        .arg("webpack")
        .arg("--config")
        .arg("webpack.std.mjs")
        .arg("--mode")
        .arg(webpack_mode)
        .output()
        .expect("failed to execute webpack");

    // package up internal modules so they'll be loaded from the tarball in-memory in runtime
    let tar_gz = File::create("modules.tar.gz").unwrap();
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    let mut modules = vec![];
    for entry in glob("src/modules_js/**/*.js").unwrap() {
        match entry {
            Ok(path) => {
                let path = path.to_str().unwrap();
                let path = path.replace("\\", "/");
                let path = path.replace("src/modules_js/", "");
                modules.push(path);
            }
            Err(e) => println!("{:?}", e),
        }
    }
    for module in modules {
        let path = format!("src/modules_js/{}", module);
        let mut file = File::open(&path).unwrap();
        let path = format!("modules/{}", module);
        tar.append_file(path, &mut file).unwrap();
    }
}
