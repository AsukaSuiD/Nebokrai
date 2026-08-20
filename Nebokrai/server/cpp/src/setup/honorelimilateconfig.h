#pragma once

#include <cstdint>
#include <filesystem>
#include <vector>

/*
 * Исходный владелец: Setup/honorelimilateconfig.cpp/.h. WorldServer использует
 * и передаёт только два первых named-параметра файла; таблица рангов имеет
 * отдельного owner-а и в этот payload не входит.
 */
class HonorElimilateConfig
{
public:
    [[nodiscard]] bool Load(const std::filesystem::path& path);
    void Serialize(std::vector<std::uint8_t>& output) const;

private:
    std::int32_t m_LevelDifference{};
    std::int32_t m_MinimumLevel{};
};
