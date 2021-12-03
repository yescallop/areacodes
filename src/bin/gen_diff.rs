use std::{fs, process::Command};

fn main() {
    let file_stems = file_name_iter().collect::<Vec<_>>();
    for pair in file_stems.windows(2) {
        Command::new("git")
            .args(["diff", "-U0", "--no-index"])
            .args([
                format!("--output=diff/{}-{}.diff", pair[0], pair[1]),
                format!("data/{}.txt", pair[0]),
                format!("data/{}.txt", pair[1]),
            ])
            .spawn()
            .unwrap();
    }
}

fn file_name_iter() -> impl Iterator<Item = String> {
    fs::read_dir("data").unwrap().map(|e| {
        let path = e.unwrap().path();
        path.file_stem().unwrap().to_string_lossy().into_owned()
    })
}
