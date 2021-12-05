use std::{
    fs::File,
    io::{BufRead, BufReader},
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
                }
                f(&buf);
                buf.clear();
            }
            Err(e) => return Err(e),
        }
    }
}
