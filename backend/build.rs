use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun this build script if .env changes
    println!("cargo:rerun-if-changed=../.env");

    // Read .env file from project root
    let env_path = Path::new("../.env");
    let env_content = fs::read_to_string(env_path)
        .expect("Failed to read .env file. Make sure .env exists in the project root.");

    // Parse .env file and set environment variables for compilation
    for line in env_content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse KEY=VALUE
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            // Set as environment variable for the build
            println!("cargo:rustc-env={}={}", key, value);
        }
    }
}
