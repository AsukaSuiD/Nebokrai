#pragma once

#include <cstdint>
#include <filesystem>
#include <map>
#include <vector>

/*
 * Исходный владелец: Setup/emotion.cpp/.h. WorldServer читает первые два
 * числа каждой `*`-записи, а GameServer получает ordered map `count,id,value`.
 * Остальные клиентские столбцы файла этому контракту не принадлежат.
 */
class CEmotion
{
public:
    [[nodiscard]] bool Load(const std::filesystem::path& path);
    [[nodiscard]] bool Serialize(std::vector<std::uint8_t>& output) const;
    [[nodiscard]] std::int32_t Repeated(std::int32_t id) const noexcept;

private:
    std::map<std::int32_t, std::int32_t> m_Emotions;
};
