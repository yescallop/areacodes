use std::{
    fs::{self, File},
    io::{BufWriter, Result, Write},
    process::Command, collections::{HashMap, hash_map::Entry},
};

fn main() -> Result<()> {
    let file_stems = file_stem_iter().collect::<Vec<_>>();
    for pair in file_stems.windows(2) {
        let res = Command::new("git")
            .args(["diff", "-U0", "--no-index"])
            .args([
                format!("data/{}.txt", pair[0]),
                format!("data/{}.txt", pair[1]),
            ])
            .output()?
            .stdout;

        if res.is_empty() {
            continue;
        }
        let res = unsafe { String::from_utf8_unchecked(res) };

        let mut lines = Vec::<&str>::new();
        let mut records = HashMap::<&str, usize>::new();

        for line in res.lines().skip(4) {
            if !line.starts_with(['@', '\\'].as_ref()) {
                match records.entry(&line[1..]) {
                    Entry::Occupied(e) => {
                        lines[*e.get()] = "";
                        continue;
                    }
                    Entry::Vacant(e) => {
                        e.insert(lines.len());
                    }
                }
                lines.push(line);
            }
        }

        let file = File::create(format!("diff/{}-{}.diff", pair[0], pair[1]))?;
        let mut bw = BufWriter::new(file);
       
        for line in lines {
            if !line.is_empty() {
                writeln!(bw, "{}", line)?;
            }
        }
    }
    Ok(())
}

fn file_stem_iter() -> impl Iterator<Item = String> {
    fs::read_dir("data").unwrap().map(|e| {
        let path = e.unwrap().path();
        path.file_stem().unwrap().to_string_lossy().into_owned()
    })
}
