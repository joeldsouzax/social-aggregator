fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "src/post.proto";

    println!("cargo:rerun-if-changed={}", proto_file);
    prost_build::compile_protos(&[proto_file], &["src/"])?;

    Ok(())
}
