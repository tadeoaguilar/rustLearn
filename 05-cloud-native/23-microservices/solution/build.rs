//! Generates Rust types, clients and servers from proto/shop.proto.
//!
//! `protox` parses .proto files in pure Rust, so there's no `protoc` to
//! install. (With `protoc` on your PATH, `tonic_prost_build::compile_protos`
//! does the same.)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto");
    let descriptors = protox::compile(["shop.proto"], ["proto"])?;
    tonic_prost_build::compile_fds(descriptors)?;
    Ok(())
}
