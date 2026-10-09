use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "compiled.gresource",
    );

    let schemas = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("schemas");
    fs::create_dir_all(&schemas).unwrap();
    fs::copy(
        "data/com.ymcx.Stocks.gschema.xml",
        schemas.join("com.ymcx.Stocks.gschema.xml"),
    )
    .unwrap();

    let status = Command::new("glib-compile-schemas")
        .arg("--strict")
        .arg(&schemas)
        .status()
        .unwrap();
    assert!(status.success(), "glib-compile-schemas failed");

    println!("cargo:rerun-if-changed=data");
}
