#include "areacodes.hpp"
#include "diff.hpp"

int main() {
    auto start = chrono::steady_clock::now();

    unordered_map<u32, Area> all_map(8192);
    unordered_map<u32, string> cur_map(4096);

    for_each_file_in(consts::DATA_DIRECTORY, [&](const auto &path) {
        string file_stem = path.stem().string();

        u32 time;
        if (from_chars(file_stem.data(), file_stem.data() + file_stem.size(),
                       time)
                .ec != errc())
            throw runtime_error(format("non-digit file stem: {}", file_stem));

        read_data(path,
                  [&](u32 code, string &&name) { cur_map[code] = move(name); });

        for (auto &[code, area] : all_map) {
            if (!cur_map.contains(code) && !area.deprecated) {
                area.entries.emplace_back(time, nullopt, nullopt);
                area.deprecated = true;
            }
        }

        for (const auto &[code, name] : cur_map) {
            optional<string_view> parent_name = nullopt;
            auto cur_it = cur_map.find(parent_code(code));
            if (cur_it != cur_map.end())
                parent_name = cur_it->second;

            auto all_it = all_map.find(code);
            if (all_it != all_map.end()) {
                Area &area = all_it->second;
                Entry &last = area.entries.back();
                if (last.name != name || last.parent_name != parent_name) {
                    area.entries.emplace_back(time, name, parent_name);
                    area.deprecated = false;
                }
            } else {
                all_map.try_emplace(code, time, name, parent_name);
            }
        }
        cur_map.clear();
        cout << format("Processed: {}\n", file_stem);
    });

    insert_diff(all_map);

    ofstream os(consts::RESULT_CSV_PATH);
    os << consts::CSV_HEADER;

    JsonEntry root = {};

    vector<u32> keys = {};
    keys.reserve(all_map.size());
    for (const auto &[key, _] : all_map) {
        keys.push_back(key);
    }
    sort(keys.begin(), keys.end());

    for (u32 code : keys) {
        const Area &area = all_map.at(code);
        const vector<Entry> &entries = area.entries;
        usize last = entries.size() - (area.deprecated ? 2 : 1);

        for (isize i = last; i >= 0; i--) {
            const Entry &entry = entries[i];
            if (!entry.name)
                continue;
            optional<u32> end = nullopt;
            if (i + 1 < entries.size())
                end = entries[i + 1].time;
            write_entry(os, root, all_map, code, *entry.name, entry.time, end,
                        i == last, entry.attr);
        }
    }

    auto ec =
        glz::write_file_json(root.children, consts::RESULT_JSON_PATH, string());
    if (ec)
        throw runtime_error("failed to write json");

    auto end = chrono::steady_clock::now();
    auto millis_elapsed =
        chrono::duration_cast<chrono::milliseconds>(end - start);
    cout << format("Finished: {}\n", millis_elapsed);

    return 0;
}

void insert_diff(unordered_map<u32, Area> &map) {
    read_fwd_diff([&](FwdDiff fd) {
        if (fd.code == 0)
            return;
        Area &area = map.at(fd.code);
        auto it = find_if(area.entries.rbegin(), area.entries.rend(),
                          [&](const auto &e) { return e.time < fd.time; });
        if (it == area.entries.rend())
            throw logic_error("entry not found");

        Entry &entry = *it;
        for (u32 code : fd.attr) {
            entry.attr.emplace(fd.optional, fd.time, code);
        }
    });

    for (auto &[code, area] : map) {
        for (usize i = 0; i < area.entries.size() - 1; i++) {
            u32 next_time = area.entries[i + 1].time;
            Entry &entry = area.entries[i];

            if (entry.attr.empty() || entry.attr.rbegin()->time != next_time) {
                entry.attr.emplace(false, next_time, code);
            }
        }
    }
}

void write_entry(ofstream &os, JsonEntry &root,
                 const unordered_map<u32, Area> &map, u32 code,
                 string_view name, u32 start, optional<u32> end, bool is_last,
                 const set<Successor> &attr) {
    JsonEntry *entry = &root;
    Level level = level_from_code(code);

    u32 prov_code = code / 10000 * 10000;
    string_view prov_name = *map.at(prov_code).entries[0].name;
    string_view pref_name = {};

    if (level != Level::Province) {
        auto it = find_if(entry->children.begin(), entry->children.end(),
                          [&](const auto &e) { return e.code == prov_code; });
        if (it == entry->children.end())
            throw logic_error("entry not found");
        entry = &*it;

        if (level == Level::Prefecture) {
            pref_name = name;
        } else {
            u32 pref_code = code / 100 * 100;
            optional<string_view> pref_name_opt = nullopt;
            const Area *area;
            if (level_from_code(pref_code) == Level::Prefecture) {
                auto area_it = map.find(pref_code);
                if (area_it != map.end()) {
                    area = &area_it->second;
                    pref_name_opt =
                        area_it->second.last_name_intersecting(start, end);
                }
            }

            if (pref_name_opt) {
                it = find_if(entry->children.begin(), entry->children.end(),
                             [&](const auto &e) {
                                 return e.code == pref_code && e.start <= start;
                             });
                if (it == entry->children.end())
                    throw logic_error("entry not found");
                entry = &*it;

                pref_name = *pref_name_opt;
            } else {
                pref_name = "直辖"sv;
            }
        }
    }

    vector<Successor> successors = {};
    for (Successor su : attr) {
        if (su.optional)
            continue;
        if (end == su.time)
            su.time = 0;
        successors.push_back(su);
    }
    entry->children.emplace_back(code, name, start, end, move(successors));

    string_view status;
    if (!end) {
        status = "启用"sv;
    } else if (is_last) {
        status = "弃用"sv;
    } else {
        status = "变更"sv;
    }
    os << format("{},{},{},{},{},{},{},", code, prov_name, pref_name, name,
                 level_description(level), status, start);
    if (end)
        os << *end;
    os << ',';

    set<pair<u32, u32>> sus = {};
    for (Successor su : attr) {
        sus.emplace(su.time, su.code);
    }
    u32 i = 0;
    for (auto [time, code] : sus) {
        if (i != 0)
            os << ';';
        os << code;
        if (end != time)
            os << '[' << time << ']';
        i += 1;
    }
    os << '\n';
}