use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    io,
};

use crate::{consts::*, files, for_each_line_in, read_data};

#[derive(Debug)]
pub struct FwdDiff<'a> {
    pub time: u32,
    pub code: u32,
    pub internal: bool,
    pub is_summary: bool,
    pub attr: &'a [u32],
    pub details_id: Option<u32>,
}

pub fn process_diff(
    mut handle_fwd_diff: impl FnMut(FwdDiff<'_>),
    mut handle_details: impl FnMut(&str),
) -> io::Result<()> {
    let mut src = DataTable::new();
    let mut dst = DataTable::new();
    let mut rem = HashMap::with_capacity(1024);
    let mut attr = Vec::new();

    let mut details_counter = 0;

    for diff in files(DIFF_DIRECTORY) {
        let file_stem = diff.file_stem().unwrap().to_str().unwrap();
        let (src_year, dst_year) = file_stem.split_once('-').unwrap();

        println!("----- {file_stem} -----");

        read_data(&format!("{DATA_DIRECTORY}/{src_year}.txt"), |code, name| {
            src.insert(code, name);
        })?;
        read_data(&format!("{DATA_DIRECTORY}/{dst_year}.txt"), |code, name| {
            dst.insert(code, name);
        })?;

        let time = dst_year.parse().unwrap();

        let mut detailed = false;
        let mut details = String::new();
        let mut details_id = None;

        for_each_line_in(&diff, |line_i, line| {
            let Ok(line) = parse_line(line) else {
                panic!("invalid line at {file_stem}.diff:{line_i}");
            };

            let line = match line {
                Line::Change(line) => {
                    if !details.is_empty() {
                        // Remove the last newline.
                        handle_details(&details[..details.len() - 1]);
                        details.clear();

                        details_id = Some(details_counter);
                        details_counter += 1;
                    }
                    line
                }
                Line::Comment(comment) => {
                    if comment == "![detailed]" {
                        detailed = true;
                    } else if detailed {
                        details.push_str(comment.trim_start());
                        details.push('\n');
                    }
                    return;
                }
                Line::Empty => {
                    details.clear();
                    details_id = None;
                    return;
                }
            };

            let code = line.code;
            let name = line.name;

            if line.internal {
                let src_name = src.name_by_code(code);
                let dst_name = dst.name_by_code(code);
                assert!(
                    src_name == dst_name && src_name == Some(name),
                    "{code}: invalid internal change",
                );
            } else if line.fwd {
                assert!(
                    src.name_by_code(code) == Some(name),
                    "{code}: invalid deletion",
                );
                // if dst.name_by_code(code) == Some(name) {
                //     println!("{code}: same-name deletion");
                // }
            } else {
                assert!(
                    dst.name_by_code(code) == Some(name),
                    "{code}: invalid addition",
                );
                // if src.name_by_code(code) == Some(name) {
                //     println!("{code}: same-name addition");
                // }
            }

            let (table, origin) = if line.fwd { (&dst, &src) } else { (&src, &dst) };

            attr.clear();
            select(table, origin, &mut rem, &line, &mut attr);

            assert!(!attr.is_empty(), "{code}: empty attr");
            let is_summary = origin.has_children(code);

            if line.fwd {
                handle_fwd_diff(FwdDiff {
                    time,
                    code,
                    internal: line.internal,
                    is_summary,
                    attr: &attr[..],
                    details_id,
                })
            } else {
                for &sel_code in &attr {
                    handle_fwd_diff(FwdDiff {
                        time,
                        code: sel_code,
                        internal: line.internal,
                        is_summary,
                        attr: &[code],
                        details_id,
                    })
                }
            }
        })?;

        for (code, rem_codes) in &rem {
            for rem_code in rem_codes {
                if rem.contains_key(rem_code) {
                    println!("{rem_code}@{code}: asymmetry found");
                }
            }
        }

        src.clear();
        dst.clear();
        rem.clear();
    }

    Ok(())
}

pub fn parent(code: u32) -> u32 {
    if code % 100 != 0 {
        code / 100 * 100
    } else if code % 10000 != 0 {
        code / 10000 * 10000
    } else {
        0
    }
}

fn code_dist(a: u32, b: u32) -> u32 {
    if a / 100 == b / 100 {
        1
    } else if a / 10000 == b / 10000 {
        2
    } else {
        3
    }
}

fn select(
    table: &DataTable,
    origin: &DataTable,
    rem: &mut HashMap<i32, HashSet<i32>>,
    line: &ChangeLine<'_>,
    res: &mut Vec<u32>,
) {
    let code = line.code;

    for sel in &line.attr {
        let mut res_code;
        match sel {
            Selector::Name { name, parent } => {
                let sel_codes = match table.codes_by_name(name) {
                    Some(codes) => codes,
                    None => panic!("{name}@{code}: not found"),
                };

                let mut min_dist = 4;
                let mut cnt = 0u32;
                res_code = 0;

                for &sel_code in sel_codes {
                    if parent.is_some()
                        && table.name_by_code(table.parent_code(sel_code)) != parent.as_deref()
                    {
                        continue;
                    }

                    let mut dist = code_dist(code, sel_code);
                    if dist == 1
                        && table.name_by_code(table.parent_code(sel_code))
                            != origin.name_by_code(origin.parent_code(sel_code))
                    {
                        dist = 2;
                    }
                    match dist.cmp(&min_dist) {
                        Ordering::Less => {
                            min_dist = dist;
                            cnt = 1;
                            res_code = sel_code;
                        }
                        Ordering::Equal => cnt += 1,
                        _ => {}
                    }
                }

                if cnt == 0 {
                    panic!("{name}@{code}: not found");
                } else if cnt != 1 {
                    panic!("{name}@{code}: multiple records found");
                }
            }
            Selector::CurCode => {
                res_code = code;
                if table.name_by_code(code).is_none() {
                    panic!("{code}: not found");
                }
            }
            Selector::ParentCode => {
                res_code = origin.parent_code(code);
                if let Some(_parent_name) = table.name_by_code(res_code) {
                    // println!("..@{code} {} = {_parent_name}", line.name);
                } else {
                    panic!("..@{code}: not found");
                }
            }
        }

        res.push(res_code);

        // Asymmetry check
        let (code, res_code) = if line.fwd {
            (-(code as i32), res_code as i32)
        } else {
            (code as i32, -(res_code as i32))
        };

        let mut insert = true;
        if let Some(rem) = rem.get_mut(&res_code) {
            if rem.remove(&code) {
                insert = false;
            }
        }
        let set = rem.entry(code).or_default();
        if insert {
            set.insert(res_code);
        }
    }
}

struct DataTable {
    c2n: HashMap<u32, String>,
    n2c: HashMap<String, Vec<u32>>,
    has_children: HashSet<u32>,
}

impl DataTable {
    fn new() -> Self {
        DataTable {
            c2n: HashMap::with_capacity(4096),
            n2c: HashMap::with_capacity(4096),
            has_children: HashSet::with_capacity(512),
        }
    }

    fn name_by_code(&self, code: u32) -> Option<&str> {
        self.c2n
            .get(&code)
            .map(|x| &**x)
            .or_else(|| (code == 0).then_some("中华人民共和国"))
    }

    fn parent_code(&self, code: u32) -> u32 {
        let code = parent(code);
        if self.c2n.contains_key(&code) {
            code
        } else {
            parent(code)
        }
    }

    fn codes_by_name(&self, name: &str) -> Option<&[u32]> {
        self.n2c.get(name).map(|x| &**x)
    }

    fn has_children(&self, code: u32) -> bool {
        self.has_children.contains(&code)
    }

    fn insert(&mut self, code: u32, name: String) {
        self.c2n.insert(code, name.clone());
        self.n2c.entry(name).or_default().push(code);
        self.has_children.insert(self.parent_code(code));
    }

    fn clear(&mut self) {
        self.c2n.clear();
        self.n2c.clear();
        self.has_children.clear();
    }
}

#[derive(Debug)]
struct ChangeLine<'a> {
    fwd: bool,
    internal: bool,
    code: u32,
    name: &'a str,
    attr: Vec<Selector<'a>>,
}

#[derive(Debug)]
enum Line<'a> {
    Change(ChangeLine<'a>),
    Comment(&'a str),
    Empty,
}

fn parse_line(line: &str) -> Result<Line<'_>, ()> {
    if line.is_empty() {
        return Ok(Line::Empty);
    }

    let mut fwd = false;
    let mut internal = false;
    match line.as_bytes()[0] {
        b'-' => fwd = true,
        b'+' => {}
        b'=' => internal = true,
        b'#' => return Ok(Line::Comment(&line[1..])),
        _ => return Err(()),
    }

    let code = line.get(1..7).and_then(|s| s.parse().ok()).ok_or(())?;
    assert_eq!(line.as_bytes()[7], b'\t');

    let line = &line[8..];
    let (name, attr_str) = line.split_once(['>', '<']).ok_or(())?;

    let actual_fwd = line.as_bytes()[name.len()] == b'>';
    if internal {
        fwd = actual_fwd;
    } else if actual_fwd != fwd {
        return Err(());
    }

    let mut attr = Vec::new();
    for mut sel in attr_str.split(',') {
        if sel.ends_with('?') {
            continue;
        }
        if let Some(stripped) = sel.strip_suffix('!') {
            sel = stripped;
        }

        let sel = match sel {
            "." => Selector::CurCode,
            ".." => Selector::ParentCode,
            _ => {
                let parent = if let Some((name, rest)) = sel.split_once('(') {
                    let Some(parent) = rest.strip_suffix(')') else {
                        return Err(());
                    };
                    sel = name;
                    Some(parent)
                } else {
                    None
                };

                if sel == "#" {
                    sel = name;
                }
                Selector::Name { name: sel, parent }
            }
        };
        attr.push(sel);
    }

    Ok(Line::Change(ChangeLine {
        fwd,
        internal,
        code,
        name,
        attr,
    }))
}

#[derive(Debug)]
enum Selector<'a> {
    Name {
        name: &'a str,
        parent: Option<&'a str>,
    },
    CurCode,
    ParentCode,
}
