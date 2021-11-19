use std::path::PathBuf;
fn main() {
    norpc::build::Compiler::default().compile("hello_world.norpc");
}
