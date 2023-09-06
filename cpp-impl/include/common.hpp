#pragma once

#include "glaze/glaze.hpp"
#include <algorithm>
#include <charconv>
#include <chrono>
#include <cstdint>
#include <filesystem>
#include <format>
#include <fstream>
#include <functional>
#include <iostream>
#include <optional>
#include <ranges>
#include <set>
#include <span>
#include <stdexcept>
#include <string>
#include <string_view>
#include <unordered_map>
#include <unordered_set>
#include <utility>
#include <variant>
#include <vector>

using std::cout;
using std::equal_to;
using std::errc;
using std::find_if;
using std::format;
using std::from_chars;
using std::function;
using std::ifstream;
using std::invalid_argument;
using std::logic_error;
using std::make_optional;
using std::make_pair;
using std::move;
using std::nullopt;
using std::ofstream;
using std::operator""sv;
using std::optional;
using std::pair;
using std::runtime_error;
using std::set;
using std::sort;
using std::span;
using std::string;
using std::string_view;
using std::tie;
using std::unordered_map;
using std::unordered_set;
using std::variant;
using std::vector;
using std::visit;
using std::views::split;

namespace chrono = std::chrono;
namespace fs = std::filesystem;

typedef std::int32_t i32;
typedef std::uint32_t u32;
typedef std::size_t usize;
typedef std::ptrdiff_t isize;

struct string_hash {
    using hash_type = std::hash<string_view>;
    using is_transparent = void;

    usize operator()(const char *str) const { return hash_type{}(str); }
    usize operator()(string_view str) const { return hash_type{}(str); }
    usize operator()(const string &str) const { return hash_type{}(str); }
};

template <class... Ts> struct overloaded : Ts... {
    using Ts::operator()...;
};
