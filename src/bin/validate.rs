use std::{collections::HashMap, io};

use areacodes::{files, for_each_line_in, read_data};

fn main() -> io::Result<()> {
    let mut src = DataTable::new();
    let mut dst = DataTable::new();

    for diff in files("diff") {
        let file_stem = diff.file_stem().unwrap().to_str().unwrap();
        let (src_year, dst_year) = file_stem.split_once('-').unwrap();

        println!("----- {file_stem} -----");

        read_data(&format!("data/{src_year}.txt"), |code, name| {
            src.insert(code, name);
        })?;
        read_data(&format!("data/{dst_year}.txt"), |code, name| {
            dst.insert(code, name);
        })?;

        for_each_line_in(&diff, |line| {
            let line = match parse_line(line) {
                Ok(Some(line)) => line,
                Ok(None) => return,
                Err(_) => panic!("invalid line in `{file_stem}.txt`: {line}"),
            };

            if line.internal {
                let src_name = src.name_by_code(line.code);
                let dst_name = dst.name_by_code(line.code);
                assert!(
                    src_name == dst_name && src_name == Some(&line.name),
                    "{}: invalid internal change",
                    line.code
                );
            } else if line.fwd {
                assert!(
                    src.name_by_code(line.code) == Some(&line.name)
                        && dst.name_by_code(line.code) != Some(&line.name),
                    "{}: invalid deletion",
                    line.code
                );
            } else {
                assert!(
                    dst.name_by_code(line.code) == Some(&line.name)
                        && src.name_by_code(line.code) != Some(&line.name),
                    "{}: invalid addition",
                    line.code
                );
            }

            let table = if line.fwd { &dst } else { &src };

            validate(&table, line);
        })?;

        src.clear();
        dst.clear();
    }

    Ok(())
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

fn validate(table: &DataTable, line: Line) {
    let code = line.code;

    for sel in line.attr {
        match &sel {
            Selector::Name { name, parent } => {
                let sel_codes = match table.codes_by_name(&name) {
                    Some(codes) => codes,
                    None => {
                        println!("{name}@{code}: not found");
                        continue;
                    }
                };

                let mut min_dist = 4;
                let mut cnt = 0u32;

                for &sel_code in sel_codes {
                    if let Some(parent) = parent {
                        if parent != table.parent_name_by_code(sel_code) {
                            continue;
                        }
                    }

                    let dist = code_dist(code, sel_code);
                    if dist < min_dist {
                        min_dist = dist;
                        cnt = 1
                    } else if dist == min_dist {
                        cnt += 1;
                    }
                }

                if cnt == 0 {
                    println!("{name}@{code}: not found");
                } else if cnt != 1 {
                    println!("{name}@{code}: multiple records found");
                }
            }
            Selector::CurCode => {
                if table.name_by_code(code).is_none() {
                    println!("{code}: not found");
                }
            }
            Selector::ParentCode => (),
        }
    }
}

struct DataTable {
    c2n: HashMap<u32, String>,
    n2c: HashMap<String, Vec<u32>>,
}

impl DataTable {
    fn new() -> Self {
        DataTable {
            c2n: HashMap::with_capacity(4096),
            n2c: HashMap::with_capacity(4096),
        }
    }

    fn name_by_code(&self, code: u32) -> Option<&str> {
        self.c2n.get(&code).map(|x| &**x)
    }

    fn parent_name_by_code(&self, code: u32) -> &str {
        fn parent(code: u32) -> u32 {
            if code % 10000 == 0 {
                0
            } else if code % 100 == 0 {
                code / 10000 * 10000
            } else {
                code / 100 * 100
            }
        }

        let code = parent(code);
        self.c2n
            .get(&code)
            .or_else(|| self.c2n.get(&parent(code)))
            .map(|x| &**x)
            .unwrap_or("中华人民共和国")
    }

    fn codes_by_name(&self, name: &str) -> Option<&[u32]> {
        self.n2c.get(name).map(|x| &**x)
    }

    fn insert(&mut self, code: u32, name: String) {
        self.c2n.insert(code, name.clone());
        self.n2c.entry(name).or_default().push(code);
    }

    fn clear(&mut self) {
        self.c2n.clear();
        self.n2c.clear();
    }
}

#[derive(Debug)]
struct Line {
    fwd: bool,
    internal: bool,
    code: u32,
    name: String,
    attr: Vec<Selector>,
}

#[derive(Debug)]
enum Selector {
    Name {
        name: String,
        parent: Option<String>,
    },
    CurCode,
    ParentCode,
}

fn parse_line(line: &str) -> Result<Option<Line>, ()> {
    if line.is_empty() {
        return Ok(None);
    }

    let mut fwd = false;
    let mut internal = false;
    match line.as_bytes()[0] {
        b'-' => fwd = true,
        b'+' => (),
        b'=' => internal = true,
        b'#' => return Ok(None),
        _ => return Err(()),
    }

    let code = line.get(1..7).and_then(|s| s.parse().ok()).ok_or(())?;
    assert_eq!(line.as_bytes()[7], b'\t');

    let line = &line[8..];
    let (name, attr) = line.split_once(['>', '<']).ok_or(())?;

    let actual_fwd = line.as_bytes()[name.len()] == b'>';
    if internal {
        fwd = actual_fwd;
    } else if actual_fwd != fwd {
        return Err(());
    }

    let mut attr_v = Vec::new();
    for mut sel in attr.split(',') {
        if sel.ends_with('?') {
            continue;
        }
        if sel.ends_with('!') {
            sel = &sel[..sel.len() - 1];
        }

        let sel = match sel {
            "." => Selector::CurCode,
            ".." => Selector::ParentCode,
            _ => {
                let parent_name = if let Some((name, rest)) = sel.split_once('(') {
                    if !rest.ends_with(')') {
                        return Err(());
                    }
                    sel = name;
                    Some(rest[..rest.len() - 1].into())
                } else {
                    None
                };

                if sel == "#" {
                    sel = name;
                }
                Selector::Name {
                    name: sel.into(),
                    parent: parent_name,
                }
            }
        };
        attr_v.push(sel);
    }

    Ok(Some(Line {
        fwd,
        internal,
        code,
        name: name.into(),
        attr: attr_v,
    }))
}
