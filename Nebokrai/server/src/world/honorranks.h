#pragma once

#include "appworld/organizingsystem/organizing.h"

#include <array>
#include <cstdint>
#include <vector>

/*
 * Исходный владелец: WorldServer/worldserver/honorranks.cpp/.h. PDB/Rust
 * подтверждают массивы [4 rank type][4 country], 0x24-byte запись и полное
 * копирование history/current в DB snapshot. MSVC lists заменены vector.
 */
struct HonorRankEntry {
    std::int32_t playerId{};
    std::uint8_t level{};
    std::array<char,20> name{};
    std::uint8_t occupation{};
    std::array<std::uint8_t,2> legacyPadding{};
    std::uint32_t appellationId{}, eliminations{};
};
using HonorRankTable = std::array<std::array<std::vector<HonorRankEntry>,4>,4>;
struct HonorRankSaveSnapshot { OrganizingTime copiedAt{}; HonorRankTable history, current; };

class CHonorRanks {
public:
    [[nodiscard]] std::vector<HonorRankEntry>& Current(std::size_t type,std::size_t country);
    [[nodiscard]] const std::vector<HonorRankEntry>& Current(std::size_t type,std::size_t country) const;
    void NewDay() { m_History=m_Current; for(auto& byCountry:m_Current)for(auto& ranks:byCountry)ranks.clear(); }
    [[nodiscard]] HonorRankSaveSnapshot GenerateSaveData(OrganizingTime now) const { return {now,m_History,m_Current}; }
private:
    HonorRankTable m_History{},m_Current{};
};
static_assert(sizeof(HonorRankEntry)==0x24);
