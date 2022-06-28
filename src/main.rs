use std::{
    collections::{hash_map::Entry::*, BTreeSet, HashMap},
    fs::File,
    io::{BufWriter, Result, Write},
    time::Instant,
};

use areacodes::*;

const DATA_DIRECTORY: &str = "data";
const RESULT_CSV_FILENAME: &str = "result.csv";
const RESULT_JSON_FILENAME: &str = "codes.json";
const CSV_HEADER: &str =
    "\u{FEFF}代码,一级行政区,二级行政区,名称,级别,状态,启用时间,变更（弃用）时间,新代码\n";

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
                    let area = e.into_mut();
                    let last = area.entries.last_mut().unwrap();
                    if last.name.as_ref() != Some(name) || last.parent_name.as_ref() != parent_name
                    {
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

    let file = File::create(RESULT_CSV_FILENAME).expect("failed to create result file");
    let mut buf = BufWriter::new(file);
    write!(buf, "{CSV_HEADER}")?;

    let mut root = JsonEntry {
        code: 0,
        name: "",
        start: 0,
        end: None,
        successors: vec![],
        children: vec![],
    };

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
                &mut root,
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

    let file = File::create(RESULT_JSON_FILENAME).expect("failed to create result file");
    serde_json::to_writer(file, &root.children).expect("failed to output json");

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
            .rev()
            .find(|e| e.time < fd.time)
            .unwrap();
        entry.attr.extend(fd.attr.iter().map(|&code| Successor {
            time: fd.time,
            code,
            opt: fd.optional,
        }));
    })?;
    for (&code, area) in map.iter_mut() {
        for i in 0..area.entries.len() - 1 {
            let next_time = area.entries[i + 1].time;
            let entry = &mut area.entries[i];
            if entry.attr.iter().rev().next().map(|su| su.time) != Some(next_time) {
                entry.attr.insert(Successor {
                    opt: false,
                    time: next_time,
                    code,
                });
            }
        }
    }
    Ok(())
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

fn write_entry<'a>(
    buf: &mut impl Write,
    root: &mut JsonEntry<'a>,
    map: &HashMap<u32, Area>,
    code: u32,
    name: &'a str,
    start: u32,
    end: Option<u32>,
    is_last: bool,
    attr: &BTreeSet<Successor>,
) -> Result<()> {
    let mut entry = root;
    let level = Level::from_code(code);

    let prov_code = code / 10000 * 10000;
    let prov_name = map[&prov_code].entries[0].name.as_deref().unwrap();
    let pref_name = if level == Level::Province {
        ""
    } else {
        entry = entry
            .children
            .iter_mut()
            .find(|e| e.code == prov_code)
            .unwrap();
        if level == Level::Prefecture {
            name
        } else {
            let pref_code = code / 100 * 100;
            let pref_name = Some(pref_code)
                .filter(|&code| Level::from_code(code) == Level::Prefecture)
                .and_then(|code| map.get(&code))
                .and_then(|area| area.last_name_intersecting(start, end));
            if let Some(name) = pref_name {
                entry = entry
                    .children
                    .iter_mut()
                    .find(|e| e.code == pref_code && e.start <= start)
                    .unwrap();
                name
            } else {
                "直辖"
            }
        }
    };

    entry.children.push(JsonEntry {
        code,
        name,
        start,
        end,
        successors: attr
            .iter()
            .copied()
            .filter(|su| !su.opt)
            .map(|mut su| {
                if end == Some(su.time) {
                    su.time = 0;
                }
                su
            })
            .collect(),
        children: vec![],
    });

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
        prov_name,
        pref_name,
        name,
        level.desc(),
        status,
        start
    )?;
    if let Some(end) = end {
        write!(buf, "{end}")?;
    }

    write!(buf, ",")?;
    let sus: BTreeSet<_> = attr.iter().map(|su| (su.time, su.code)).collect();
    for (i, &(time, code)) in sus.iter().enumerate() {
        if i != 0 {
            write!(buf, ";")?;
        }
        write!(buf, "{}", code)?;
        if end != Some(time) {
            write!(buf, "[{}]", time)?;
        }
    }

    writeln!(buf)
}

#[derive(PartialEq, Eq)]
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
    attr: BTreeSet<Successor>,
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
