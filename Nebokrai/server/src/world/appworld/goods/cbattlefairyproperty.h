#pragma once

#include <cstdint>
#include <string>
#include <utility>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/goods/cbattlefairyproperty.cpp/.h.
 * PDB подтверждает compose и level записи. Старый AddToByteArray_Combine
 * копировал MSVC string ABI целиком (0x7C), поэтому он не воспроизводится как
 * переносимый wire-формат: owner отдаёт логические записи внешнему совмести-
 * мому codec-у клиента. Singleton заменён owned registry.
 */
struct BattleFairyCompose {
    std::string fetchStone, fetchBody, material;
    std::uint32_t depleteFetch{};
    float successRate{};
    std::string battleFairy;
    std::uint32_t index{};
};
struct BattleFairyLevel { std::int32_t equipmentLevel{}; float successRate{}; std::int32_t upValue{}, totalValue{}; };

class CBattleFairyProperty {
public:
    void AddCompose(BattleFairyCompose value) { m_Compose.push_back(std::move(value)); }
    void AddLevel(BattleFairyLevel value) { m_Levels.push_back(value); }
    [[nodiscard]] const std::vector<BattleFairyCompose>& Compose() const noexcept { return m_Compose; }
    [[nodiscard]] const std::vector<BattleFairyLevel>& Levels() const noexcept { return m_Levels; }
    [[nodiscard]] const BattleFairyLevel* FindLevel(std::int32_t equipmentLevel) const noexcept;
private:
    std::vector<BattleFairyCompose> m_Compose;
    std::vector<BattleFairyLevel> m_Levels;
};
