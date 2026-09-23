use std::path::Path;
use std::process::Command;

fn main() {
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "compiled.gresource",
    );

    // glib-build-tools has no schema helper, so compile the schemas ourselves.
    compile_schemas(Path::new("data"));

    println!("cargo:rerun-if-changed=data/icon.svg");
    println!("cargo:rerun-if-changed=data/com.ymcx.Stocks.gschema.xml");
    // Track the generated file too, so deleting it forces a rerun.
    println!("cargo:rerun-if-changed=data/gschemas.compiled");
}

fn compile_schemas(dir: &Path) {
    let output = dir.join("gschemas.compiled");

    // Skip if the output exists and is newer than every schema. Rewriting it
    // unconditionally would change its mtime and make Cargo rerun forever.
    if !needs_compile(dir, &output) {
        return;
    }

    let status = Command::new("glib-compile-schemas")
        .arg(dir)
        .status()
        .expect("failed to run glib-compile-schemas");

    assert!(status.success(), "glib-compile-schemas failed");
}

fn needs_compile(dir: &Path, output: &Path) -> bool {
    let Ok(output_mtime) = std::fs::metadata(output).and_then(|meta| meta.modified()) else {
        return true;
    };

    let newest_schema = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".gschema.xml"))
        .filter_map(|entry| entry.metadata().ok()?.modified().ok())
        .max();

    newest_schema.is_some_and(|schema_mtime| schema_mtime > output_mtime)
}
