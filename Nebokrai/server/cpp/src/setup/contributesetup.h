#pragma once

#include <array>
#include <cstdint>
#include <filesystem>
#include <string>
#include <vector>

/*
 * Исходный владелец: Setup/contributesetup.cpp/.h. Подтверждены одиннадцать
 * positional параметров, `# lo hi name count` и wire в том же порядке.
 * Файловая граница и владение данными локальны, старые globals удалены.
 */
class CContributeSetup
{
public:
    struct Item { std::uint32_t low{}, high{}, count{}; std::string name; };

    [[nodiscard]] bool Load(const std::filesystem::path& path);
    [[nodiscard]] bool Serialize(std::vector<std::uint8_t>& output) const;

private:
    std::array<std::int32_t, 11> m_Parameters{};
    std::vector<Item> m_Items;
};
