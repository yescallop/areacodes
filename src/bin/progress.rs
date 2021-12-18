use areacodes::*;
use std::{
    fs::{self, File},
    path::PathBuf,
};

fn main() -> Result<()> {
    let (mut total, mut finished) = (0u32, 0u32);
    for path in diff_dir_iter() {
        let file = File::open(&path)?;
        for_each_line_in(file, |line| {
            if line.starts_with(['-', '+', '='].as_ref()) {
                total += 1;
                if line.contains(['>', '<'].as_ref()) {
                    finished += 1;
                }
            }
        })?;
    }
    println!(
        "{}/{} ({:.1}%)",
        finished,
        total,
        finished as f32 / total as f32 * 100.0
    );
    Ok(())
}

fn diff_dir_iter() -> impl Iterator<Item = PathBuf> {
    fs::read_dir("diff")
        .expect("failed to read diff directory")
        .map(|e| e.unwrap().path())
}
