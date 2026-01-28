fn main() -> anyhow::Result<()> {
    // Recompile if proto file changes
    println!("cargo:rerun-if-changed=proto/blog.proto");

    // Generate only client code
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&["proto/blog.proto"], &["proto/"])?;

    Ok(())
}
