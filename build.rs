use std::process::Command;

fn main() {
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "compiled.gresource",
    );

    let status = Command::new("glib-compile-schemas")
        .arg("data")
        .status()
        .unwrap();
    assert!(status.success());

    println!("cargo:rerun-if-changed=data");
}
