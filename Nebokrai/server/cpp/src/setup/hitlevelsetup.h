#pragma once

#include <cstdint>
#include <filesystem>
#include <vector>

/*
 * Исходный владелец: Setup/hitlevelsetup.cpp/.h из WorldServer и GameServer.
 * PDB/EXE подтверждают записи по три DWORD и wire `count + raw records`.
 * Старое process-global хранилище заменено обычным value-owner.
 */
class CHitLevelSetup
{
public:
    struct Entry { std::uint32_t level{}, hit{}, experience{}; };

    [[nodiscard]] bool Load(const std::filesystem::path& path);
    [[nodiscard]] bool Serialize(std::vector<std::uint8_t>& output) const;
    [[nodiscard]] const std::vector<Entry>& Entries() const noexcept { return m_Entries; }

private:
    std::vector<Entry> m_Entries;
};

static_assert(sizeof(CHitLevelSetup::Entry) == 0x0C);
