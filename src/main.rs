use std::collections::{hash_map::Entry::*, HashMap};
use std::fs::{self, File};
use std::io::{BufWriter, Result, Write};
use std::path::PathBuf;
use std::time::Instant;

use areacodes::*;

const DATA_DIRECTORY: &str = "data";
const RESULT_FILENAME: &str = "result.csv";
const CSV_HEADER: &str =
    "\u{FEFF}代码,一级行政区,二级行政区（变更前）,名称,级别,状态,启用时间（含）,弃用时间（不含）\n";

fn main() -> Result<()> {
    let start = Instant::now();

    let mut all_map = HashMap::<u32, Area>::with_capacity(8192);
    let mut cur_map = HashMap::<u32, String>::with_capacity(4096);

    for path in data_dir_iter() {
        let file_stem = path
            .file_stem()
            .expect("no file name")
            .to_str()
            .expect("invalid file stem");

        let time: u32 = file_stem.parse().expect("non-digit file stem");

        let file = File::open(&path).expect("failed to open file");

        for_each_line_in(file, |line| {
            let code = line
                .get(0..6)
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| panic!("invalid line in `{}.txt`: {}", file_stem, line));
            let name = line[7..].to_owned();

            cur_map.insert(code, name);
        })?;

        for (code, area) in &mut all_map {
            if !cur_map.contains_key(code) && !area.deprecated {
                area.entries.push(Entry { time, name: None });
                area.deprecated = true;
            }
        }

        for (code, name) in cur_map.drain() {
            let name = Some(name);
            match all_map.entry(code) {
                Occupied(e) => {
                    let area = e.into_mut();
                    if area.entries.last().unwrap().name != name {
                        area.entries.push(Entry { time, name })
                    }
                    area.deprecated = false;
                }
                Vacant(e) => {
                    e.insert(Area::new(Entry { time, name }));
                }
            }
        }
        println!("Processed: {}", file_stem);
    }

    let file = File::create(RESULT_FILENAME).expect("failed to create result file");
    let mut buf = BufWriter::new(file);
    write!(buf, "{}", CSV_HEADER)?;

    let mut keys = all_map.keys().copied().collect::<Vec<_>>();
    keys.sort_unstable();

    for code in keys {
        let area = &all_map[&code];
        let entries = &area.entries;
        let last = entries.len() - if area.deprecated { 2 } else { 1 };
        for i in (0..=last).rev() {
            let entry = &entries[i];
            let name = match entry.name.as_deref() {
                Some(name) => name,
                None => continue,
            };
            let end = entries.get(i + 1).map(|e| e.time);
            write_entry(&mut buf, &all_map, code, name, entry.time, end, i == last)?;
        }
    }
    buf.flush()?;

    println!("Finished: {:?}", start.elapsed());
    Ok(())
}

fn write_entry(
    buf: &mut impl Write,
    map: &HashMap<u32, Area>,
    code: u32,
    name: &str,
    start: u32,
    end: Option<u32>,
    is_last: bool,
) -> Result<()> {
    let level = Level::from_code(code);

    let province = map[&(code / 10000 * 10000)].entries[0]
        .name
        .as_deref()
        .unwrap();
    let prefecture = match level {
        Level::PREFECTURE => name,
        Level::COUNTY => map
            .get(&(code / 100 * 100))
            .and_then(|area| area.last_name_intersecting(start, end))
            .unwrap_or("直管"),
        Level::PROVINCE => "",
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
    )?;
    if let Some(end) = end {
        write!(buf, "{}", end)?;
    }
    writeln!(buf)
}

fn data_dir_iter() -> impl Iterator<Item = PathBuf> {
    fs::read_dir(DATA_DIRECTORY)
        .expect("failed to read data directory")
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
            Level::PROVINCE => "省级",
            Level::PREFECTURE => "地级",
            Level::COUNTY => "县级",
        }
    }

    fn from_code(code: u32) -> Level {
        if code % 100 != 0 {
            Level::COUNTY
        } else if code % 10000 != 0 {
            Level::PREFECTURE
        } else {
            Level::PROVINCE
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

    fn last_name_intersecting(&self, start: u32, end: Option<u32>) -> Option<&str> {
        let last = self.entries.len() - 1;
        let end = match end {
            Some(end) => end,
            None => return self.entries[last].name.as_deref(),
        };

        for i in (0..=last).rev() {
            let cur = &self.entries[i];
            if i == last && !self.deprecated {
                if cur.time < end {
                    return cur.name.as_deref();
                }
                continue;
            }
            if cur.name.is_none() {
                continue;
            }
            if self.entries[i + 1].time > start && cur.time < end {
                return cur.name.as_deref();
            }
        }
        None
    }
}

struct Entry {
    time: u32,
    name: Option<String>,
}
