use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema_path = get_proto_schema();
    println!("Searching for schemas in: {}", schema_path.display());
    let proto_file = schema_path.join("post.proto");
    println!("cargo:rerun-if-changed={}", proto_file.display());

    if !proto_file.exists() {
        println!("post proto does not exists. Skipping.");
        return Ok(());
    }
    prost_build::compile_protos(
        &[proto_file.to_str().unwrap()],  // The full path to your proto file
        &[schema_path.to_str().unwrap()], // The directory to search for imports
    )?;

    Ok(())
}

fn get_proto_schema() -> PathBuf {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let project_root = manifest_dir.parent().unwrap().parent().unwrap();
    project_root.join("schemas")
}
