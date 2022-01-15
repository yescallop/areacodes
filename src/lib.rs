use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};

pub use std::io::Result;

pub fn for_each_line_in(file: File, mut f: impl FnMut(&str)) -> Result<()> {
    let mut br = BufReader::new(file);
    let mut buf = String::with_capacity(64);

    loop {
        match br.read_line(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(_n) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                f(&buf);
                buf.clear();
            }
            Err(e) => return Err(e),
        }
    }
}

pub fn files(path: &str) -> impl Iterator<Item = PathBuf> {
    fs::read_dir("diff")
        .unwrap_or_else(|_| panic!("failed to read directory: {}", path))
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
}
