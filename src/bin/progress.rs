use areacodes::{consts::*, *};

fn main() -> Result<()> {
    let (mut total, mut finished) = (0u32, 0u32);
    for path in files(DIFF_DIRECTORY) {
        for_each_line_in(path, |_, line| {
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
