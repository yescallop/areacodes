use areacodes::*;
use std::fs::File;

fn main() -> Result<()> {
    let (mut total, mut finished) = (0u32, 0u32);
    for path in files("diff") {
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
