use std::io::Result;

fn main() -> Result<()> {
    // Compile proto files
    prost_build::Config::new()
        .out_dir("src/generated")
        .compile_protos(
            &["proto/api.proto"],
            &["proto"],
        )?;

    // Tell cargo to re-run if proto files change
    println!("cargo:rerun-if-changed=proto/api.proto");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
