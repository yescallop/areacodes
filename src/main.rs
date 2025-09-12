use std::{
    collections::{BTreeMap, BTreeSet, HashMap, hash_map::Entry::*},
    fs::File,
    io::{BufWriter, Result, Seek, Write},
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
        sql_codes: BufWriter::new(File::create(OUTPUT_SQL_CODES_PATH)?),
        sql_changes: BufWriter::new(File::create(OUTPUT_SQL_CHANGES_PATH)?),
        sql_descriptions: BufWriter::new(File::create(OUTPUT_SQL_DESCRIPTIONS_PATH)?),
    };
    write!(out.csv, "{CSV_HEADER}")?;
    write!(out.sql_codes, "{SQL_CODES_HEADER}")?;
    write!(out.sql_changes, "{SQL_CHANGES_HEADER}")?;
    write!(out.sql_descriptions, "{SQL_DESCRIPTIONS_HEADER}")?;

    for (i, text) in out.json.descriptions.iter().enumerate() {
        if i != 0 {
            writeln!(out.sql_descriptions, ",")?;
        }
        write!(out.sql_descriptions, "('{text}')")?;
    }
    writeln!(out.sql_descriptions, ";\nSET @id = LAST_INSERT_ID();\n")?;

    let mut desc_map = BTreeMap::new();

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
                &mut desc_map,
            )?;
        }
    }

    let bw = BufWriter::new(File::create(OUTPUT_JSON_PATH)?);
    serde_json::to_writer(bw, &out.json).expect("failed to write JSON data");

    for (id, rows) in desc_map.into_iter() {
        if rows.len() == 1 {
            let id = if id == 0 { "@id" } else { "@id := @id + 1" };
            let (code, new_code, time) = rows[0];
            writeln!(
                out.sql_descriptions,
                "UPDATE `changes` SET `desc_id` = {id} WHERE (`code`, `new_code`, `time`) = ({code}, {new_code}, {time});",
            )?;
        } else {
            if id != 0 {
                writeln!(out.sql_descriptions, "SET @id = @id + 1;")?;
            }
            write!(
                out.sql_descriptions,
                "UPDATE `changes` SET `desc_id` = @id WHERE (`code`, `new_code`, `time`) IN (",
            )?;
            for (i, (code, new_code, time)) in rows.into_iter().enumerate() {
                if i != 0 {
                    write!(out.sql_descriptions, ", ")?;
                }
                write!(out.sql_descriptions, "({code}, {new_code}, {time})")?;
            }
            writeln!(out.sql_descriptions, ");")?;
        }
    }

    writeln!(out.sql_codes, ";")?;
    writeln!(out.sql_changes, ";")?;
    writeln!(out.sql_descriptions, "COMMIT;")?;

    println!("Finished: {:?}", start.elapsed());
    Ok(())
}

fn insert_diff(map: &mut HashMap<u32, Area>) -> Result<Vec<String>> {
    let mut descriptions = vec![];
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
                desc_id: fd.desc_id,
            }));
        },
        |text| descriptions.push(text.into()),
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
    sql_codes: BufWriter<File>,
    sql_changes: BufWriter<File>,
    sql_descriptions: BufWriter<File>,
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
    desc_map: &mut BTreeMap<u32, Vec<(u32, u32, u32)>>,
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
        successors: attr
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

    if out.sql_codes.stream_position()? != SQL_CODES_HEADER.len() as u64 {
        writeln!(out.sql_codes, ",")?;
    }

    write!(out.sql_codes, "({code}, '{name}', {start}, ")?;
    match end {
        Some(end) => write!(out.sql_codes, "{end})")?,
        None => write!(out.sql_codes, "NULL)")?,
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

    attr.iter().try_for_each(|su| {
        if out.sql_changes.stream_position()? != SQL_CHANGES_HEADER.len() as u64 {
            writeln!(out.sql_changes, ",")?;
        }

        if let Some(id) = su.desc_id {
            desc_map
                .entry(id)
                .or_default()
                .push((code, su.code, su.time));
        }
        write!(out.sql_changes, "({code}, {}, {})", su.code, su.time)
    })?;

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
