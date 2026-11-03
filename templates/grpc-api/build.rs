// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Compiles `proto/` (the generated `.proto` files and your own) into the
//! messages, the servers and the clients, with protox (a protobuf compiler
//! in Rust) in place of `protoc`. The descriptors also go to
//! `$OUT_DIR/file_descriptor_set.bin`, which the reflection service serves.

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut protos: Vec<PathBuf> = std::fs::read_dir("proto")?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<_, _>>()?;
    protos.retain(|path| path.extension().is_some_and(|ext| ext == "proto"));
    protos.sort();
    let mut compiler = protox::Compiler::new(["proto"])?;
    compiler
        .include_imports(true)
        .include_source_info(true)
        .open_files(&protos)?;
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("cargo sets OUT_DIR")?);
    std::fs::write(
        out_dir.join("file_descriptor_set.bin"),
        compiler.encode_file_descriptor_set(),
    )?;
    tonic_prost_build::configure().compile_fds(compiler.file_descriptor_set())?;
    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
