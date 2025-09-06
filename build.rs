use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=TARIUM_EMBED_GITHUB_APP_ID");
    println!("cargo:rerun-if-env-changed=TARIUM_EMBED_GITHUB_INSTALLATION_ID");
    println!("cargo:rerun-if-env-changed=TARIUM_EMBED_GITHUB_PRIVATE_KEY_PATH");
    println!("cargo:rerun-if-env-changed=TARIUM_EMBED_GITHUB_PRIVATE_KEY");

    // Check if we should embed GitHub App credentials
    let should_embed = env::var("TARIUM_EMBED_CREDENTIALS").unwrap_or_default() == "1"
        || env::var("TARIUM_EMBED_GITHUB_APP_ID").is_ok();

    if should_embed {
        embed_github_credentials();
    } else {
        // If not embedding, we still need to provide dummy values for the env! macros
        println!("cargo:rustc-env=TARIUM_EMBEDDED_APP_ID=");
        println!("cargo:rustc-env=TARIUM_EMBEDDED_INSTALLATION_ID=");
        println!("cargo:rustc-env=TARIUM_EMBEDDED_PRIVATE_KEY=");
    }
}

fn embed_github_credentials() {
    println!("cargo:warning=Embedding GitHub App credentials into binary");

    // Get App ID
    let app_id = env::var("TARIUM_EMBED_GITHUB_APP_ID")
        .expect("TARIUM_EMBED_GITHUB_APP_ID must be set when embedding credentials");

    // Get Installation ID
    let installation_id = env::var("TARIUM_EMBED_GITHUB_INSTALLATION_ID")
        .expect("TARIUM_EMBED_GITHUB_INSTALLATION_ID must be set when embedding credentials");

    // Get Private Key (either from file path or directly)
    let private_key = if let Ok(key_path) = env::var("TARIUM_EMBED_GITHUB_PRIVATE_KEY_PATH") {
        if !Path::new(&key_path).exists() {
            panic!("Private key file not found: {}", key_path);
        }

        fs::read_to_string(&key_path)
            .unwrap_or_else(|e| panic!("Failed to read private key from {}: {}", key_path, e))
    } else if let Ok(key_content) = env::var("TARIUM_EMBED_GITHUB_PRIVATE_KEY") {
        key_content
    } else {
        panic!("Either TARIUM_EMBED_GITHUB_PRIVATE_KEY_PATH or TARIUM_EMBED_GITHUB_PRIVATE_KEY must be set");
    };

    // Validate private key format
    if !private_key.contains("BEGIN") || !private_key.contains("PRIVATE KEY") {
        panic!("Private key does not appear to be in PEM format");
    }

    // Validate App ID is numeric
    if app_id.parse::<u64>().is_err() {
        panic!("App ID must be numeric: {}", app_id);
    }

    // Validate Installation ID is numeric
    if installation_id.parse::<u64>().is_err() {
        panic!("Installation ID must be numeric: {}", installation_id);
    }

    // Set environment variables for compilation
    println!("cargo:rustc-env=TARIUM_EMBEDDED_APP_ID={}", app_id);
    println!(
        "cargo:rustc-env=TARIUM_EMBEDDED_INSTALLATION_ID={}",
        installation_id
    );

    // For the private key, we need to escape it properly for embedding
    let escaped_key = private_key
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");

    println!(
        "cargo:rustc-env=TARIUM_EMBEDDED_PRIVATE_KEY={}",
        escaped_key
    );

    println!("cargo:warning=GitHub App credentials embedded successfully");
    println!("cargo:warning=App ID: {}", app_id);
    println!("cargo:warning=Installation ID: {}", installation_id);
    println!(
        "cargo:warning=Private key: {} characters",
        private_key.len()
    );
}
