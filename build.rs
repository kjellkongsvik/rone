fn main() {
    prost_build::compile_protos(&["protos/core.proto"], &["protos/"]).unwrap();
}
