#include "areacodes.hpp"

bool operator<(const Successor &l, const Successor &r) {
    return tie(l.optional, l.time, l.code) < tie(r.optional, r.time, r.code);
}

Entry::Entry(u32 time, optional<string_view> name,
             optional<string_view> parent_name)
    : time(time), name(move(copied(name))),
      parent_name(move(copied(parent_name))), attr() {}

JsonEntry::JsonEntry(u32 code, string_view name, u32 start, optional<u32> end,
                     vector<Successor> &&successors)
    : code(code), name(name), start(start), end(end),
      successors(move(successors)), children() {}

optional<string_view> Area::last_name_intersecting(u32 start,
                                                   optional<u32> end) const {
    usize last = entries.size() - 1;
    if (!end)
        return entries[last].name;

    for (isize i = last; i >= 0; i--) {
        const Entry &cur = entries[i];
        if (i == last && !deprecated) {
            if (cur.time < *end)
                return cur.name;
            continue;
        }
        if (!cur.name)
            continue;
        if (entries[i + 1].time > start && cur.time < *end)
            return cur.name;
    }
    return nullopt;
}

Area::Area(u32 time, optional<string_view> name,
           optional<string_view> parent_name)
    : entries(), deprecated(false) {
    entries.emplace_back(time, name, parent_name);
}

u32 parent_code(u32 code) {
    if (code % 100 != 0) {
        return code / 100 * 100;
    } else if (code % 10000 != 0) {
        return code / 10000 * 10000;
    } else {
        return 0;
    }
}

u32 code_distance(u32 a, u32 b) {
    if (a / 100 == b / 100) {
        return 1;
    } else if (a / 10000 == b / 10000) {
        return 2;
    } else {
        return 3;
    }
}

Level level_from_code(u32 code) {
    if (code % 100 != 0) {
        return Level::County;
    } else if (code % 10000 != 0) {
        return Level::Prefecture;
    } else {
        return Level::Province;
    }
}

string_view level_description(Level level) {
    switch (level) {
    case Level::Province:
        return "省级"sv;
    case Level::Prefecture:
        return "地级"sv;
    case Level::County:
        return "县级"sv;
    default:
        throw logic_error("invalid enum variant");
    }
}

optional<pair<string_view, string_view>> split_once(string_view str, char ch) {
    usize i = str.find(ch);
    if (i == string_view::npos)
        return nullopt;
    return pair(str.substr(0, i), str.substr(i + 1));
}

optional<pair<string_view, string_view>> split_once(string_view str,
                                                    const char *s) {
    usize i = str.find_first_of(s);
    if (i == string_view::npos)
        return nullopt;
    return pair(str.substr(0, i), str.substr(i + 1));
}

optional<string> copied(optional<string_view> opt) {
    return opt ? make_optional(string(*opt)) : nullopt;
}

void for_each_file_in(const fs::path &path,
                      function<void(const fs::path &)> f) {
    set<fs::path> files = {};
    for (const auto &entry : fs::directory_iterator(path)) {
        if (entry.is_regular_file())
            files.insert(entry.path());
    }
    for (const auto &path : files) {
        f(path);
    }
}

void read_data(const fs::path &path, function<void(u32, string &&)> f) {
    string filename = path.filename().string();

    ifstream is(path);
    string line;
    u32 line_i = 0;
    while (getline(is, line)) {
        line_i += 1;
        if (line.size() < 7)
            throw runtime_error(
                format("{}({}): line too short", filename, line_i));
        u32 code;
        if (from_chars(line.data(), line.data() + 6, code).ec != errc())
            throw runtime_error(
                format("{}({}): invalid code", filename, line_i));
        if (line[6] != '\t')
            throw runtime_error(format("{}({}): no tab", filename, line_i));
        f(code, move(line.substr(7)));
    }
}
