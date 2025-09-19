use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap, HashSet},
    io,
};

use crate::{consts::*, files, for_each_line_in, read_data};

#[derive(Debug)]
pub struct FwdDiff<'a> {
    pub time: u32,
    pub code: u32,
    pub transfer: bool,
    pub attr: &'a [u32],
    pub desc_id: Option<u32>,
}

pub fn process_diff(
    mut handle_fwd_diff: impl FnMut(FwdDiff<'_>),
    mut handle_description: impl FnMut(u32, &str),
) -> io::Result<()> {
    let mut src = DataTable::new();
    let mut dst = DataTable::new();
    let mut rem = HashMap::with_capacity(1024);
    let mut attr = Vec::new();

    // Check for possible omission or duplication by calculating
    // common codes twice and comparing two results.
    let mut codes_src = BTreeSet::new();
    let mut codes_dst = BTreeSet::new();

    for diff in files(DIFF_DIRECTORY) {
        let file_stem = diff.file_stem().unwrap().to_str().unwrap();
        let (src_year, dst_year) = file_stem.split_once('-').unwrap();

        println!("----- {file_stem} -----");

        read_data(&format!("{DATA_DIRECTORY}/{src_year}.txt"), |code, name| {
            src.insert(code, name);
            codes_src.insert(code);
        })?;
        read_data(&format!("{DATA_DIRECTORY}/{dst_year}.txt"), |code, name| {
            dst.insert(code, name);
            codes_dst.insert(code);
        })?;

        let time = dst_year.parse().unwrap();

        let mut described = false;
        let mut desc = String::new();
        let mut desc_id = None;
        let mut desc_counter = 0;

        for_each_line_in(&diff, |line_i, line| {
            let line = parse_line(line)
                .unwrap_or_else(|| panic!("invalid line at {file_stem}.diff:{line_i}"));

            let line = match line {
                Line::Change(line) => {
                    if !desc.is_empty() {
                        // Remove the last newline.
                        handle_description(time, &desc[..desc.len() - 1]);
                        desc.clear();

                        desc_id = Some(desc_counter);
                        desc_counter += 1;
                    }
                    line
                }
                Line::Comment(comment) => {
                    if comment == "![described]" {
                        described = true;
                    } else if described {
                        desc.push_str(comment.trim_start());
                        desc.push('\n');
                    }
                    return;
                }
                Line::Empty => {
                    desc.clear();
                    desc_id = None;
                    return;
                }
            };

            let code = line.code;
            let name = line.name;

            assert!(!described || desc_id.is_some(), "{code}: no description");

            if line.transfer {
                let src_name = src.name_by_code(code);
                let dst_name = dst.name_by_code(code);
                assert!(
                    src_name == dst_name && src_name == Some(name),
                    "{code}: invalid transfer",
                );
            } else if line.fwd {
                assert!(
                    src.name_by_code(code) == Some(name),
                    "{code}: invalid deletion",
                );
                assert!(codes_src.remove(&code), "{code}: duplicate deletion");
            } else {
                assert!(
                    dst.name_by_code(code) == Some(name),
                    "{code}: invalid addition",
                );
                assert!(codes_dst.remove(&code), "{code}: duplicate addition");
            }

            let (table, origin) = if line.fwd { (&dst, &src) } else { (&src, &dst) };

            attr.clear();
            select(table, origin, &mut rem, &line, &mut attr);

            let has_children = origin.has_children(code);
            if has_children {
                assert!(attr.is_empty(), "{code}: nonempty attr with children");
                return;
            } else {
                assert!(!attr.is_empty(), "{code}: empty attr without children");
            }

            if line.fwd {
                handle_fwd_diff(FwdDiff {
                    time,
                    code,
                    transfer: line.transfer,
                    attr: &attr[..],
                    desc_id,
                })
            } else {
                for &sel_code in &attr {
                    handle_fwd_diff(FwdDiff {
                        time,
                        code: sel_code,
                        transfer: line.transfer,
                        attr: &[code],
                        desc_id,
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

        let sym_diff: BTreeSet<_> = codes_src.symmetric_difference(&codes_dst).collect();
        if !sym_diff.is_empty() {
            panic!("omission detected: {sym_diff:?}")
        }

        src.clear();
        dst.clear();
        rem.clear();
        codes_src.clear();
        codes_dst.clear();
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

    for sel in line.attr.iter().flatten() {
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

        let insert = !rem.get_mut(&res_code).is_some_and(|rem| rem.remove(&code));
        let set = rem.entry(code).or_default();
        if insert {
            set.insert(res_code);
        }
    }
}

struct DataTable {
    names_by_code: HashMap<u32, String>,
    codes_by_name: HashMap<String, Vec<u32>>,
    codes_with_children: HashSet<u32>,
}

impl DataTable {
    fn new() -> Self {
        DataTable {
            names_by_code: HashMap::with_capacity(4096),
            codes_by_name: HashMap::with_capacity(4096),
            codes_with_children: HashSet::with_capacity(512),
        }
    }

    fn name_by_code(&self, code: u32) -> Option<&str> {
        self.names_by_code
            .get(&code)
            .map(|x| &**x)
            .or_else(|| (code == 0).then_some("中华人民共和国"))
    }

    fn parent_code(&self, code: u32) -> u32 {
        let code = parent(code);
        if self.names_by_code.contains_key(&code) {
            code
        } else {
            parent(code)
        }
    }

    fn codes_by_name(&self, name: &str) -> Option<&[u32]> {
        self.codes_by_name.get(name).map(|x| &**x)
    }

    fn has_children(&self, code: u32) -> bool {
        self.codes_with_children.contains(&code)
    }

    fn insert(&mut self, code: u32, name: String) {
        self.names_by_code.insert(code, name.clone());
        self.codes_by_name.entry(name).or_default().push(code);
        self.codes_with_children.insert(self.parent_code(code));
    }

    fn clear(&mut self) {
        self.names_by_code.clear();
        self.codes_by_name.clear();
        self.codes_with_children.clear();
    }
}

#[derive(Debug)]
struct ChangeLine<'a> {
    fwd: bool,
    transfer: bool,
    code: u32,
    name: &'a str,
    attr: Option<Vec<Selector<'a>>>,
}

#[derive(Debug)]
enum Line<'a> {
    Change(ChangeLine<'a>),
    Comment(&'a str),
    Empty,
}

fn parse_line(line: &str) -> Option<Line<'_>> {
    if line.is_empty() {
        return Some(Line::Empty);
    }

    let mut fwd = false;
    let mut transfer = false;
    match line.as_bytes()[0] {
        b'-' => fwd = true,
        b'+' => {}
        b'=' => transfer = true,
        b'#' => return Some(Line::Comment(&line[1..])),
        _ => return None,
    }

    if line.len() < 8 {
        return None;
    }

    let code = line[1..7].parse().ok()?;

    if line.as_bytes()[7] != b' ' {
        return None;
    }

    let line = &line[8..];
    let (name, attr) = match line.split_once(['>', '<']) {
        Some((name, attr)) => {
            let actual_fwd = line.as_bytes()[name.len()] == b'>';
            if transfer {
                fwd = actual_fwd;
            } else if actual_fwd != fwd {
                return None;
            }
            (name, Some(parse_attr(attr, name)?))
        }
        None => {
            if transfer {
                return None;
            }
            (line, None)
        }
    };

    Some(Line::Change(ChangeLine {
        fwd,
        transfer,
        code,
        name,
        attr,
    }))
}

fn parse_attr<'a>(attr: &'a str, name: &'a str) -> Option<Vec<Selector<'a>>> {
    let mut res = Vec::new();
    for mut sel in attr.split(',') {
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
                    sel = name;
                    Some(rest.strip_suffix(')')?)
                } else {
                    None
                };

                if sel == "#" {
                    sel = name;
                } else if sel == name {
                    println!("unnecessary repetition: {name}");
                }
                Selector::Name { name: sel, parent }
            }
        };
        res.push(sel);
    }
    Some(res)
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
