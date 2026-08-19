#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <filesystem>
#include <utility>
#include <vector>

/*
 * Owner: authserver/src/kl_ipfilter.h
 *
 * Точная пара: AuthServer/authserver.exe + AuthServer/authserver.pdb; исходный
 * owner PDB: h:\fengyun\fy_russia\src\server\authserver\src\kl_ipfilter.h.
 * EXE/PDB подтверждает точную wildcard-семантику kl_net::ip_filter:
 * нулевой octet именно сохранённого правила означает wildcard, остальные
 * octet обязаны совпадать. Allow-вариант разрешает совпадение, deny-вариант —
 * его отсутствие. Порядок правил сохраняется линейным списком.
 *
 * Canonical file parser принимает четыре decimal octet 0..255, включая
 * ведущие нули. Старый atoi принимал также знак/prefix и затем сужал значение;
 * эта CRT-граница не доказана и возвращается отдельным статусом, а не получает
 * придуманную современную трактовку.
 */
namespace kl_net
{
using IpPattern = std::array<std::uint8_t, 4>;

enum class IpFilterLoadStatus
{
    Ok,
    IoError,
    LegacyAtoiBoundaryUnknown,
};

struct IpFilterLoadResult
{
    IpFilterLoadStatus status{IpFilterLoadStatus::IoError};
    std::vector<IpPattern> patterns;
    std::size_t tokenIndex{};

    [[nodiscard]] constexpr explicit operator bool() const noexcept
    {
        return status == IpFilterLoadStatus::Ok;
    }
};

[[nodiscard]] IpFilterLoadResult LoadIpPatterns(const std::filesystem::path& path);

template <bool AllowMatch>
class IpFilter
{
public:
    void ReplacePatterns(std::vector<IpPattern> patterns)
    {
        m_Patterns = std::move(patterns);
    }

    [[nodiscard]] bool IsAllowed(const IpPattern& address) const noexcept
    {
        bool matched = false;
        for (const IpPattern& pattern : m_Patterns) {
            bool current = true;
            for (std::size_t index = 0; index < pattern.size(); ++index) {
                if (pattern[index] != 0U && pattern[index] != address[index]) {
                    current = false;
                    break;
                }
            }
            if (current) {
                matched = true;
                break;
            }
        }
        return matched == AllowMatch;
    }

    [[nodiscard]] const std::vector<IpPattern>& Patterns() const noexcept
    {
        return m_Patterns;
    }

private:
    std::vector<IpPattern> m_Patterns;
};
}
