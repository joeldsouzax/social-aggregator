fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "src/post.proto";

    println!("cargo:rerun-if-changed={}", proto_file);
    prost_build::Config::new()
        .type_attribute("social.v1.Post", "#[derive(serde::Serialize)]")
        .field_attribute(
            "social.v1.Post.timestamp",
            "#[serde(with = \"crate::prost_timestamp_serde\")]",
        )
        .type_attribute("social.v1.PostBatch", "#[derive(serde::Serialize)]")
        .compile_protos(&[proto_file], &["src/"])?;
    Ok(())
}
