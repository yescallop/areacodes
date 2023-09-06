#pragma once

#include "common.hpp"

namespace consts {

const char DATA_DIRECTORY[] = "../data";
const char DIFF_DIRECTORY[] = "../diff";
const char RESULT_CSV_PATH[] = "../result.csv";
const char RESULT_JSON_PATH[] = "../codes.json";
const char CSV_HEADER[] = "\uFEFF代码,一级行政区,二级行政区,名称,级别,状态,"
                          "启用时间,变更（弃用）时间,新代码\n";

} // namespace consts

struct Successor {
    bool optional;
    u32 time;
    u32 code;
};

template <> struct glz::meta<Successor> {
    using T = Successor;
    static constexpr auto value = object(
        "optional",
        [](auto &&self) {
            return self.optional ? make_optional(true) : nullopt;
        },
        "time",
        [](auto &&self) {
            return self.time != 0 ? make_optional(self.time) : nullopt;
        },
        "code", &T::code);
};

bool operator<(const Successor &l, const Successor &r);

struct Entry {
    u32 time;
    optional<string> name;
    optional<string> parent_name;
    set<Successor> attr;

    Entry(u32 time, optional<string_view> name,
          optional<string_view> parent_name);
};

struct JsonEntry {
    u32 code;
    string_view name;
    u32 start;
    optional<u32> end;
    vector<Successor> successors;
    vector<JsonEntry> children;

    JsonEntry() = default;
    JsonEntry(u32 code, string_view name, u32 start, optional<u32> end,
              vector<Successor> &&successors);
};

template <> struct glz::meta<JsonEntry> {
    using T = JsonEntry;
    static constexpr auto value = object(
        "code", &T::code, "name", &T::name, "start", &T::start, "end", &T::end,
        "successors",
        [](auto &&self) {
            return !self.successors.empty()
                       ? make_optional(::span(self.successors))
                       : nullopt;
        },
        "children",
        [](auto &&self) {
            return !self.children.empty() ? make_optional(::span(self.children))
                                          : nullopt;
        });
};

struct Area {
    vector<Entry> entries;
    bool deprecated;

    Area(u32 time, optional<string_view> name,
         optional<string_view> parent_name);

    optional<string_view> last_name_intersecting(u32 start,
                                                 optional<u32> end) const;
};

enum Level {
    Province,
    Prefecture,
    County,
};

u32 parent_code(u32 code);
u32 code_distance(u32 a, u32 b);
Level level_from_code(u32 code);
string_view level_description(Level level);
optional<pair<string_view, string_view>> split_once(string_view str, char ch);
optional<pair<string_view, string_view>> split_once(string_view str,
                                                    const char *s);
optional<string> copied(optional<string_view> opt);
void for_each_file_in(const fs::path &path, function<void(const fs::path &)> f);
void read_data(const fs::path &path, function<void(u32, string &&)> f);
void write_entry(ofstream &os, JsonEntry &root,
                 const unordered_map<u32, Area> &map, u32 code,
                 string_view name, u32 start, optional<u32> end, bool is_last,
                 const set<Successor> &attr);