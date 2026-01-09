use std::io::Result;

fn main() -> Result<()> {
    prost_build::Config::new()
        .out_dir("src/generated")
        .compile_protos(
            &["../proto/api.proto"],
            &["../proto"],
        )?;

    println!("cargo:rerun-if-changed=../proto/api.proto");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
