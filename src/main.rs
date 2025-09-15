use std::{
    collections::{BTreeMap, BTreeSet, HashMap, hash_map::Entry::*},
    fs::File,
    io::{BufWriter, Result, Write},
    time::Instant,
};

use areacodes::{consts::*, *};

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
                    let last: &mut Entry = area.entries.last_mut().unwrap();
                    let parent_name_changed = last.parent_name.as_ref() != parent_name;

                    if last.name.as_ref() != Some(name) || parent_name_changed {
                        last.parent_name_changed = parent_name_changed;
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

    let mut out = Output {
        csv: BufWriter::new(File::create(OUTPUT_CSV_PATH)?),
        json: JsonOutput {
            items: vec![],
            descriptions: insert_diff(&mut all_map)?,
        },
    };
    write!(out.csv, "{CSV_HEADER}")?;

    let mut keys = all_map.keys().copied().collect::<Vec<_>>();
    keys.sort_unstable();

    for code in keys {
        let area = &all_map[&code];
        let entries = &area.entries;
        let last = entries.len() - if area.deprecated { 2 } else { 1 };
        for i in (0..=last).rev() {
            let entry = &entries[i];
            let Some(name) = entry.name.as_deref() else {
                continue;
            };
            let end = entries.get(i + 1).map(|e| e.time);
            write_entry(
                &mut out,
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

    let bw = BufWriter::new(File::create(OUTPUT_JSON_PATH)?);
    serde_json::to_writer_pretty(bw, &out.json).expect("failed to write JSON data");

    println!("Finished: {:?}", start.elapsed());
    Ok(())
}

fn insert_diff(map: &mut HashMap<u32, Area>) -> Result<BTreeMap<u32, Vec<String>>> {
    let mut descriptions = BTreeMap::<u32, Vec<String>>::new();
    process_diff(
        |fd| {
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
                desc: fd.desc_id,
            }));
        },
        |time, text| descriptions.entry(time).or_default().push(text.into()),
    )?;

    for area in map.values() {
        for i in 0..area.entries.len() - 1 {
            let end = area.entries[i + 1].time;
            let entry = &area.entries[i];
            if entry.name.is_some()
                && entry.parent_name_changed
                && entry.attr.last().map(|su| su.time) != Some(end)
            {
                println!("{entry:?}: parent name changed with no corresponding diff");
            }
        }
    }
    Ok(descriptions)
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

struct Output<'a> {
    csv: BufWriter<File>,
    json: JsonOutput<'a>,
}

#[allow(clippy::too_many_arguments)]
fn write_entry<'a>(
    out: &mut Output<'a>,
    map: &HashMap<u32, Area>,
    code: u32,
    name: &'a str,
    start: u32,
    end: Option<u32>,
    is_last: bool,
    attr: &BTreeSet<Successor>,
) -> Result<()> {
    let mut items = &mut out.json.items;
    let level = Level::from_code(code);

    let prov_code = code / 10000 * 10000;
    let prov_name = map[&prov_code].entries[0].name.as_deref().unwrap();
    let pref_name = if level == Level::Province {
        ""
    } else {
        items = &mut items
            .iter_mut()
            .find(|e| e.code == prov_code)
            .unwrap()
            .children;
        if level == Level::Prefecture {
            name
        } else {
            let pref_code = code / 100 * 100;
            let pref_name = Some(pref_code)
                .filter(|&code| Level::from_code(code) == Level::Prefecture)
                .and_then(|code| map.get(&code))
                .and_then(|area| area.last_name_intersecting(start, end));
            if let Some(name) = pref_name {
                items = &mut items
                    .iter_mut()
                    .find(|e| e.code == pref_code && e.start <= start)
                    .unwrap()
                    .children;
                name
            } else {
                "直辖"
            }
        }
    };

    items.push(CodeItem {
        code,
        name,
        start,
        end,
        succ: attr
            .iter()
            .copied()
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
        "在用"
    } else if is_last {
        "弃用"
    } else {
        "变更"
    };
    write!(
        out.csv,
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
        write!(out.csv, "{end}")?;
    }

    write!(out.csv, ",")?;
    let sus: BTreeSet<_> = attr.iter().map(|su| (su.time, su.code)).collect();
    for (i, &(time, new_code)) in sus.iter().enumerate() {
        if i != 0 {
            write!(out.csv, ";")?;
        }
        write!(out.csv, "{}", new_code)?;
        if end != Some(time) {
            write!(out.csv, "[{}]", time)?;
        }
    }

    writeln!(out.csv)
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
        let Some(end) = end else {
            return self.entries[last].name.as_deref();
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
    parent_name_changed: bool,
}

impl Entry {
    fn new(time: u32, name: Option<&String>, parent_name: Option<&String>) -> Entry {
        Entry {
            time,
            name: name.cloned(),
            parent_name: parent_name.cloned(),
            attr: BTreeSet::new(),
            parent_name_changed: false,
        }
    }
}
