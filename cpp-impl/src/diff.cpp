#include "diff.hpp"

GivenName GivenName::parse(string_view cur_name, string_view sel_str) {
    optional<string_view> parent = nullopt;
    auto parts = split_once(sel_str, '(');
    if (parts) {
        auto [name, rest] = *parts;
        if (!rest.ends_with(')'))
            throw invalid_argument("invalid selector");
        rest.remove_suffix(1);
        parent = rest;
        sel_str = name;
    }

    if (sel_str == "#")
        sel_str = cur_name;
    return {sel_str, parent};
}

DataTable::DataTable()
    : code_to_name(4096), name_to_code(4096), _has_children(512) {}

optional<string_view> DataTable::name_by_code(u32 code) const {
    auto it = code_to_name.find(code);
    if (it != code_to_name.end())
        return it->second;
    if (code == 0)
        return "中华人民共和国"sv;
    return nullopt;
}

u32 DataTable::parent_code_exact(u32 code) const {
    code = parent_code(code);
    return code_to_name.contains(code) ? code : parent_code(code);
}

optional<span<const u32>> DataTable::codes_by_name(string_view name) const {
    auto it = name_to_code.find(name);
    if (it != name_to_code.end())
        return it->second;
    return nullopt;
}

bool DataTable::has_children(u32 code) const {
    return _has_children.contains(code);
}

void DataTable::insert(u32 code, string &&name) {
    code_to_name[code] = name;
    name_to_code.try_emplace(move(name)).first->second.push_back(code);
    _has_children.insert(parent_code_exact(code));
}

void DataTable::clear() {
    code_to_name.clear();
    name_to_code.clear();
    _has_children.clear();
}

optional<Line> Line::parse(string_view line) {
    if (line.empty())
        return nullopt;

    bool fwd = false, internal = false;
    switch (line[0]) {
    case '-':
        fwd = true;
        break;
    case '+':
        break;
    case '=':
        internal = true;
        break;
    case '#':
        return nullopt;
    default:
        throw invalid_argument("invalid line");
    }

    if (line.size() < 8)
        throw invalid_argument("invalid line");

    u32 code;
    if (from_chars(line.data() + 1, line.data() + 7, code).ec != errc())
        throw invalid_argument("invalid line");

    line = line.substr(8);
    auto parts = split_once(line, "><");
    if (!parts)
        throw invalid_argument("invalid line");
    auto [name, attr_str] = *parts;

    bool actual_fwd = line[name.size()] == '>';
    if (internal) {
        fwd = actual_fwd;
    } else if (actual_fwd != fwd)
        throw invalid_argument("invalid line");

    vector<Selector> attr = {};
    for (const auto range : split(attr_str, ',')) {
        string_view sel_str(range);
        if (sel_str.ends_with('?'))
            continue;
        if (sel_str.ends_with('!'))
            sel_str.remove_suffix(1);

        Selector sel;
        if (sel_str == "."sv) {
            sel = CurCode();
        } else if (sel_str == ".."sv) {
            sel = ParentCode();
        } else {
            sel = GivenName::parse(name, sel_str);
        }
        attr.push_back(sel);
    }
    return Line(fwd, internal, code, name, attr);
}

void read_fwd_diff(function<void(FwdDiff)> f) {
    DataTable src = {};
    DataTable dst = {};
    unordered_map<i32, unordered_set<i32>> rem(1024);
    vector<u32> attr = {};

    for_each_file_in(consts::DIFF_DIRECTORY, [&](const auto &path) {
        string file_stem = path.stem().string();
        auto parts = split_once(file_stem, '-');
        if (!parts)
            throw runtime_error(format("{}: invalid file stem", file_stem));
        auto [src_year, dst_year] = *parts;

        cout << format("----- {} -----\n", file_stem);

        read_data(
            format("{}/{}.txt", consts::DATA_DIRECTORY, src_year),
            [&](u32 code, string &&name) { src.insert(code, move(name)); });
        read_data(
            format("{}/{}.txt", consts::DATA_DIRECTORY, dst_year),
            [&](u32 code, string &&name) { dst.insert(code, move(name)); });

        u32 time;
        if (from_chars(dst_year.data(), dst_year.data() + dst_year.size(), time)
                .ec != errc())
            throw runtime_error(format("invalid time: {}", dst_year));

        ifstream is(path);
        string line_str;
        u32 line_i = 0;
        while (getline(is, line_str)) {
            line_i += 1;
            Line line;
            try {
                optional<Line> line_opt = Line::parse(line_str);
                if (!line_opt)
                    continue;
                line = *line_opt;
            } catch (invalid_argument) {
                throw runtime_error(format("line {}: invalid format", line_i));
            }

            u32 code = line.code;
            string_view name = line.name;

            if (line.internal) {
                auto src_name = src.name_by_code(code);
                auto dst_name = dst.name_by_code(code);
                if (src_name != dst_name || src_name != name)
                    throw runtime_error(
                        format("{}: invalid internal change", code));
            } else if (line.fwd) {
                if (src.name_by_code(code) != name)
                    throw runtime_error(format("{}: invalid deletion", code));
                if (dst.name_by_code(code) == name)
                    cout << format("{}: same-name deletion\n", code);
            } else {
                if (dst.name_by_code(code) != name)
                    throw runtime_error(format("{}: invalid addition", code));
                if (src.name_by_code(code) == name)
                    cout << format("{}: same-name addition\n", code);
            }

            const DataTable &table = line.fwd ? dst : src;
            const DataTable &origin = line.fwd ? src : dst;

            attr.clear();
            select(table, origin, rem, line, attr);

            if (attr.empty())
                throw runtime_error(format("{}: empty attr", code));
            bool optional = origin.has_children(code);

            if (line.fwd) {
                f({time, code, line.internal, optional, attr});
            } else {
                for (u32 sel_code : attr) {
                    f({time, sel_code, line.internal, optional,
                       span(&code, 1)});
                }
            }
        }

        for (const auto &[code, rem_codes] : rem) {
            for (i32 rem_code : rem_codes) {
                if (rem.contains(rem_code))
                    cout << format("{}@{}: asymmetry found\n", rem_code, code);
            }
        }

        src.clear();
        dst.clear();
        rem.clear();
    });
}

void select(const DataTable &table, const DataTable &origin,
            unordered_map<i32, unordered_set<i32>> &rem, const Line &line,
            vector<u32> &res) {
    u32 code = line.code;

    for (Selector sel : line.attr) {
        u32 res_code;

        auto given_name = [&](GivenName sel) {
            auto [name, parent] = sel;
            auto sel_codes = table.codes_by_name(name);
            if (!sel_codes)
                throw runtime_error(format("{}@{}: not found", name, code));

            u32 min_dist = 4;
            u32 cnt = 0;
            res_code = 0;

            for (u32 sel_code : *sel_codes) {
                if (parent && table.name_by_code(table.parent_code_exact(
                                  sel_code)) != parent) {
                    continue;
                }

                u32 dist = code_distance(code, sel_code);
                if (dist == 1 &&
                    table.name_by_code(table.parent_code_exact(sel_code)) !=
                        origin.name_by_code(
                            origin.parent_code_exact(sel_code))) {
                    dist = 2;
                }

                if (dist < min_dist) {
                    min_dist = dist;
                    cnt = 1;
                    res_code = sel_code;
                } else if (dist == min_dist) {
                    cnt += 1;
                }
            }

            if (cnt == 0) {
                throw runtime_error(format("{}@{}: not found", name, code));
            } else if (cnt != 1) {
                throw runtime_error(
                    format("{}@{}: multiple records found", name, code));
            }
        };

        auto cur_code = [&](CurCode) {
            res_code = code;
            if (!table.name_by_code(code)) {
                throw runtime_error(format("{}: not found", code));
            }
        };

        auto parent_code = [&](ParentCode) {
            res_code = origin.parent_code_exact(code);
            auto parent_name = table.name_by_code(res_code);
            if (parent_name) {
                // cout << format("..@{} {} = {}\n", code, line.name,
                //                *parent_name);
            } else {
                throw runtime_error(format("..@{}: not found", code));
            }
        };

        visit(overloaded{given_name, cur_code, parent_code}, sel);

        res.push_back(res_code);

        // Asymmetry check
        i32 i_code = -i32(code), i_res_code = i32(res_code);
        if (!line.fwd) {
            i_code = -i_code;
            i_res_code = -i_res_code;
        }

        bool insert = true;
        auto it = rem.find(res_code);
        if (it != rem.end()) {
            if (it->second.erase(code))
                insert = false;
        }

        unordered_set<i32> &rem_set = rem.try_emplace(code).first->second;
        if (insert)
            rem_set.insert(res_code);
    }
}