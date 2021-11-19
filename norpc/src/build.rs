use std::fs::File;
use std::path::{Path, PathBuf};

pub struct Compiler {}
impl Compiler {
    pub fn default() -> Self {
        Self {}
    }
    pub fn compile<P: AsRef<Path>>(self, filepath: P) {
        use std::io::Write;

        // compile
        let s = std::fs::read_to_string(filepath.as_ref()).unwrap();
        let code = crate::compiler::compile(&s);
        dbg!(&s);

        // output
        let basename = filepath.as_ref().file_name().unwrap();
        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let mut out_file = out_dir;
        out_file.push(basename);
        out_file.set_extension("rs");
        dbg!(&out_file);
        let mut f = File::create(out_file).unwrap();
        f.write_all(code.as_bytes()).unwrap();
    }
}
