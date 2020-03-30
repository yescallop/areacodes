use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use rustc_hash::FxHashMap;

use crate::Level::*;

const DATA_DIRECTORY: &str = "data";
const RESULT_FILENAME: &str = "result.csv";
const CSV_HEADER: &str =
    "\u{FEFF}代码,一级行政区,二级行政区（变更前）,名称,级别,状态,启用时间,弃用时间\n";

fn main() {
    let mut all_map = FxHashMap::<u32, Area>::with_capacity_and_hasher(8192, Default::default());
    let mut cur_map = FxHashMap::<u32, String>::with_capacity_and_hasher(4096, Default::default());

    for path in data_path_iter() {
        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let time: u32 = file_stem.parse().unwrap();
        let file = File::open(&path).unwrap();

        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            let code = line[0..6].parse().unwrap();
            let name = line[7..].to_owned();
            cur_map.insert(code, name);
        }

        for (code, area) in &mut all_map {
            if !cur_map.contains_key(code) && !area.deprecated {
                area.entries.push(Entry { time, name: None });
                area.deprecated = true;
            }
        }

        for (code, name) in cur_map.drain() {
            let name = Some(name);
            if let Some(area) = all_map.get_mut(&code) {
                if area.entries.last().unwrap().name != name {
                    area.entries.push(Entry { time, name })
                }
                area.deprecated = false;
            } else {
                all_map.insert(code, Area::new(Entry { time, name }));
            }
        }
        println!("Processed: {}", file_stem);
    }

    let mut buf = BufWriter::new(File::create(RESULT_FILENAME).unwrap());
    buf.write(CSV_HEADER.as_bytes()).unwrap();

    let mut keys = all_map.keys().collect::<Vec<_>>();
    keys.sort_unstable();
    for code in keys {
        let area = &all_map[code];
        let entries = &area.entries;
        let last = entries.len() - if area.deprecated { 2 } else { 1 };
        for i in (0..=last).rev() {
            let entry = &entries[i];
            if entry.name.is_none() {
                continue;
            }
            let name = entry.name.as_ref().unwrap();
            let end = entries.get(i + 1).map(|e| e.time);
            write_entry(&mut buf, &all_map, *code, name, entry.time, end, i == last);
        }
    }
    buf.flush().unwrap();
}

fn write_entry(
    buf: &mut impl Write,
    map: &FxHashMap<u32, Area>,
    code: u32,
    name: &str,
    start: u32,
    end: Option<u32>,
    is_last: bool,
) {
    let level = Level::from_code(code);

    let province = map[&(code / 10000 * 10000)].entries[0]
        .name
        .as_ref()
        .unwrap();
    let prefecture = match level {
        PREFECTURE => name,
        COUNTY => match map.get(&(code / 100 * 100)) {
            Some(area) => area.last_name_intersecting(start, end).map(|s| s.as_str()),
            None => None,
        }
        .unwrap_or("直管"),
        PROVINCE => "",
    };

    let status = if end.is_none() {
        "启用"
    } else if is_last {
        "弃用"
    } else {
        "变更"
    };
    write!(
        buf,
        "{},{},{},{},{},{},{},",
        code,
        province,
        prefecture,
        name,
        level.desc(),
        status,
        start
    )
    .unwrap();
    if let Some(end) = end {
        write!(buf, "{}", end).unwrap();
    }
    buf.write(&[b'\n']).unwrap();
}

fn data_path_iter() -> impl Iterator<Item = PathBuf> {
    fs::read_dir(DATA_DIRECTORY)
        .unwrap()
        .map(|e| e.unwrap().path())
}

enum Level {
    PROVINCE,
    PREFECTURE,
    COUNTY,
}

impl Level {
    fn desc(&self) -> &str {
        match self {
            PROVINCE => "省级",
            PREFECTURE => "地级",
            COUNTY => "县级",
        }
    }

    fn from_code(code: u32) -> Level {
        if code % 100 != 0 {
            COUNTY
        } else if code % 10000 != 0 {
            PREFECTURE
        } else {
            PROVINCE
        }
    }
}

struct Area {
    entries: Vec<Entry>,
    deprecated: bool,
}

impl Area {
    fn new(entry: Entry) -> Area {
        Area {
            entries: vec![entry],
            deprecated: false,
        }
    }

    fn last_name_intersecting(&self, start: u32, end: Option<u32>) -> Option<&String> {
        let last = self.entries.len() - 1;
        if end.is_none() {
            return self.entries[last].name.as_ref();
        }
        let end = end.unwrap();
        for i in (0..=last).rev() {
            let cur = &self.entries[i];
            if i == last && !self.deprecated {
                if cur.time < end {
                    return cur.name.as_ref();
                }
                continue;
            }
            if cur.name.is_none() {
                continue;
            }
            if self.entries[i + 1].time > start && cur.time < end {
                return cur.name.as_ref();
            }
        }
        None
    }
}

struct Entry {
    time: u32,
    name: Option<String>,
}
