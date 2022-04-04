use std::{
    collections::{hash_map::Entry::*, BTreeSet, HashMap},
    fs::File,
    io::{BufWriter, Result, Write},
    time::Instant,
};

use areacodes::*;

const DATA_DIRECTORY: &str = "data";
const RESULT_FILENAME: &str = "result.csv";
const CSV_HEADER: &str =
    "\u{FEFF}代码,一级行政区,二级行政区,名称,级别,状态,启用时间,弃用时间,新代码\n";

fn main() -> Result<()> {
    let start = Instant::now();

    let mut all_map = HashMap::<u32, Area>::with_capacity(8192);
    let mut cur_map = HashMap::<u32, String>::with_capacity(4096);

    for path in files(DATA_DIRECTORY) {
        let file_stem = path
            .file_stem()
            .expect("no file name")
            .to_str()
            .expect("invalid file stem");

        let time: u32 = file_stem.parse().expect("non-digit file stem");

        read_data(&path, |code, name| {
            cur_map.insert(code, name);
        })?;

        for (code, area) in &mut all_map {
            if !cur_map.contains_key(code) && !area.deprecated {
                area.entries.push(Entry::new(time, None, None));
                area.deprecated = true;
            }
        }

        for (&code, name) in &cur_map {
            let parent_name = parent_name(&cur_map, code);
            match all_map.entry(code) {
                Occupied(e) => {
                    let last = e.get().entries.last().unwrap();
                    if last.name.as_ref() != Some(name) || last.parent_name.as_ref() != parent_name
                    {
                        let area = e.into_mut();
                        area.entries.push(Entry::new(time, Some(name), parent_name));
                        area.deprecated = false;
                    }
                }
                Vacant(e) => {
                    e.insert(Area::new(Entry::new(time, Some(name), parent_name)));
                }
            }
        }
        cur_map.clear();
        println!("Processed: {file_stem}");
    }

    insert_diff(&mut all_map)?;

    let file = File::create(RESULT_FILENAME).expect("failed to create result file");
    let mut buf = BufWriter::new(file);
    write!(buf, "{CSV_HEADER}")?;

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
            write_entry(
                &mut buf,
                &all_map,
                code,
                name,
                entry.time,
                end,
                i == last,
                &entry.attr,
            )?;
        }
    }
    buf.flush()?;

    println!("Finished: {:?}", start.elapsed());
    Ok(())
}

fn insert_diff(map: &mut HashMap<u32, Area>) -> Result<()> {
    for_each_fwd_diff(|fd| {
        if fd.code == 0 {
            return;
        }
        let area = map.get_mut(&fd.code).unwrap();
        let entry = area
            .entries
            .iter_mut()
            .take_while(|e| e.time < fd.time)
            .last()
            .unwrap();
        entry.attr.extend(fd.attr.iter().filter(|&&x| {
            let x = parent(x);
            x != fd.code && parent(x) != fd.code
        }));
    })
}

fn parent_name(map: &HashMap<u32, String>, code: u32) -> Option<&String> {
    let code = if code % 100 != 0 {
        code / 100 * 100
    } else if code % 10000 != 0 {
        code / 10000 * 10000
    } else {
        0
    };
    map.get(&code)
}

fn write_entry(
    buf: &mut impl Write,
    map: &HashMap<u32, Area>,
    code: u32,
    name: &str,
    start: u32,
    end: Option<u32>,
    is_last: bool,
    attr: &BTreeSet<u32>,
) -> Result<()> {
    let level = Level::from_code(code);

    let province = map[&(code / 10000 * 10000)].entries[0]
        .name
        .as_deref()
        .unwrap();
    let prefecture = match level {
        Level::Prefecture => name,
        Level::County => Some(code / 100 * 100)
            .filter(|&code| matches!(Level::from_code(code), Level::Prefecture))
            .and_then(|code| map.get(&code))
            .and_then(|area| area.last_name_intersecting(start, end))
            .unwrap_or("直辖"),
        Level::Province => "",
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
        write!(buf, "{end}")?;
    }

    write!(buf, ",")?;
    for (i, &new_code) in attr.iter().enumerate() {
        if i != 0 {
            write!(buf, ";")?;
        } else if attr.len() == 1 && new_code == code {
            break;
        }
        write!(buf, "{new_code}")?;
    }

    writeln!(buf)
}

enum Level {
    Province,
    Prefecture,
    County,
}

impl Level {
    fn desc(&self) -> &str {
        match self {
            Level::Province => "省级",
            Level::Prefecture => "地级",
            Level::County => "县级",
        }
    }

    fn from_code(code: u32) -> Level {
        if code % 100 != 0 {
            Level::County
        } else if code % 10000 != 0 {
            Level::Prefecture
        } else {
            Level::Province
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
struct Entry {
    time: u32,
    name: Option<String>,
    parent_name: Option<String>,
    attr: BTreeSet<u32>,
}

impl Entry {
    fn new(time: u32, name: Option<&String>, parent_name: Option<&String>) -> Entry {
        Entry {
            time,
            name: name.map(Clone::clone),
            parent_name: parent_name.map(Clone::clone),
            attr: BTreeSet::new(),
        }
    }
}
