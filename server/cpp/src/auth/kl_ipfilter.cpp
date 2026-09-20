#include "kl_ipfilter.h"

#include <fstream>
#include <iterator>
#include <optional>
#include <string>

namespace kl_net
{
namespace
{
std::optional<IpPattern> ParseCanonicalPattern(const std::string& token)
{
    IpPattern result{};
    std::size_t cursor = 0;
    for (std::size_t part = 0; part < result.size(); ++part) {
        if (cursor >= token.size()) {
            return std::nullopt;
        }

        std::uint32_t value = 0;
        std::size_t digits = 0;
        while (cursor < token.size() && token[cursor] != '.') {
            const unsigned char ch = static_cast<unsigned char>(token[cursor]);
            if (ch < '0' || ch > '9') {
                return std::nullopt;
            }
            value = value * 10U + static_cast<std::uint32_t>(ch - '0');
            if (value > 255U) {
                return std::nullopt;
            }
            ++digits;
            ++cursor;
        }
        if (digits == 0U) {
            return std::nullopt;
        }
        result[part] = static_cast<std::uint8_t>(value);

        if (part + 1U == result.size()) {
            if (cursor != token.size()) {
                return std::nullopt;
            }
        } else {
            if (cursor >= token.size() || token[cursor] != '.') {
                return std::nullopt;
            }
            ++cursor;
        }
    }
    return result;
}
}

IpFilterLoadResult LoadIpPatterns(const std::filesystem::path& path)
{
    std::ifstream input(path, std::ios::binary);
    if (!input) {
        return {IpFilterLoadStatus::IoError, {}, 0};
    }

    std::vector<IpPattern> patterns;
    std::string token;
    std::size_t tokenIndex = 0;
    while (input >> token) {
        const auto pattern = ParseCanonicalPattern(token);
        if (!pattern) {
            return {IpFilterLoadStatus::LegacyAtoiBoundaryUnknown,
                    std::move(patterns),
                    tokenIndex};
        }
        patterns.push_back(*pattern);
        ++tokenIndex;
    }
    if (input.bad()) {
        return {IpFilterLoadStatus::IoError, {}, tokenIndex};
    }
    return {IpFilterLoadStatus::Ok, std::move(patterns), tokenIndex};
}
}
