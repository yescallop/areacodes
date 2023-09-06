#![warn(rust_2018_idioms)]

use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

pub use std::io::Result;

pub mod consts {
    pub const DATA_DIRECTORY: &str = "data";
    pub const DIFF_DIRECTORY: &str = "diff";
    pub const RESULT_CSV_PATH: &str = "result.csv";
    pub const RESULT_JSON_PATH: &str = "codes.json";
    pub const CSV_HEADER: &str =
        "\u{FEFF}代码,一级行政区,二级行政区,名称,级别,状态,启用时间,变更（弃用）时间,新代码\n";
}

mod diff;
pub use diff::*;

#[derive(serde::Serialize, Default)]
pub struct JsonEntry<'a> {
    pub code: u32,
    pub name: &'a str,
    pub start: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub successors: Vec<Successor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<JsonEntry<'a>>,
}

#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Successor {
    #[serde(skip_serializing_if = "is_default")]
    pub optional: bool,
    #[serde(skip_serializing_if = "is_default")]
    pub time: u32,
    pub code: u32,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    *t == T::default()
}

pub fn for_each_line_in(path: impl AsRef<Path>, mut f: impl FnMut(&str)) -> Result<()> {
    let file = File::open(path)?;
    let mut br = BufReader::new(file);
    let mut buf = String::with_capacity(64);

    while br.read_line(&mut buf)? != 0 {
        if buf.ends_with('\n') {
            buf.pop();
            if buf.ends_with('\r') {
                buf.pop();
            }
        }
        f(&buf);
        buf.clear();
    }
    Ok(())
}

pub fn files(path: &str) -> impl Iterator<Item = PathBuf> {
    let files: BTreeSet<_> = fs::read_dir(path)
        .unwrap_or_else(|_| panic!("failed to read directory: {path}"))
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
        .collect();
    files.into_iter()
}

pub fn read_data(path: &impl AsRef<Path>, mut f: impl FnMut(u32, String)) -> Result<()> {
    let file_name = path.as_ref().file_name().unwrap().to_str().unwrap();
    for_each_line_in(path, |line| {
        let code = line
            .get(0..6)
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| panic!("invalid line in `{file_name}`: {line}"));
        assert_eq!(line.as_bytes()[6], b'\t');
        f(code, line[7..].into());
    })
}
