#pragma once

#include "areacodes.hpp"
#include "common.hpp"

struct FwdDiff {
    u32 time;
    u32 code;
    bool internal;
    bool optional;
    span<u32> attr;
};

struct GivenName {
    string_view name;
    optional<string_view> parent;

    static GivenName parse(string_view cur_name, string_view sel_str);
};
struct CurCode {};
struct ParentCode {};

using Selector = variant<GivenName, CurCode, ParentCode>;

class DataTable {
    unordered_map<u32, string> code_to_name;
    unordered_map<string, vector<u32>, string_hash, equal_to<>> name_to_code;
    unordered_set<u32> _has_children;

  public:
    DataTable();
    optional<string_view> name_by_code(u32 code) const;
    u32 parent_code_exact(u32 code) const;
    optional<span<const u32>> codes_by_name(string_view name) const;
    bool has_children(u32 code) const;
    void insert(u32 code, string &&name);
    void clear();
};

struct Line {
    bool fwd;
    bool internal;
    u32 code;
    string_view name;
    vector<Selector> attr;

    static optional<Line> parse(string_view line);
};

void read_fwd_diff(function<void(FwdDiff)> f);
void select(const DataTable &table, const DataTable &origin,
            unordered_map<i32, unordered_set<i32>> &rem, const Line &line,
            vector<u32> &res);
void insert_diff(unordered_map<u32, Area> &map);
