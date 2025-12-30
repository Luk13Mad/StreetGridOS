fn main() {
    prost_build::compile_protos(&["../proto/neighborhood.proto"], &["../proto/"]).unwrap();
}
