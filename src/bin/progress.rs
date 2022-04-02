use areacodes::*;

fn main() -> Result<()> {
    let (mut total, mut finished) = (0u32, 0u32);
    for path in files("diff") {
        for_each_line_in(&path, |line| {
            if line.starts_with(['-', '+', '=']) {
                total += 1;
                if line.contains(['>', '<']) {
                    finished += 1;
                }
            }
        })?;
    }
    println!(
        "{finished}/{total} ({:.1}%)",
        finished as f32 / total as f32 * 100.0
    );
    Ok(())
}
