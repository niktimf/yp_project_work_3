fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Recompile if proto file changes
    println!("cargo:rerun-if-changed=proto/blog.proto");

    // Generate both server and client code
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/blog.proto"], &["proto/"])?;

    Ok(())
}
