#pragma once

#include <cstdint>
#include <deque>
#include <map>
#include <optional>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/organizingsystem/factionwarsys.cpp/.h.
 * Типы войны, unordered-пары врагов, обновление первой пары и delayed expiry
 * подтверждены Rust-реконструкцией/EXE. Внешние изменения faction и рассылка
 * выполняются владельцем controller через возвращаемые завершённые пары.
 */
struct FactionWarType { std::int32_t type{}, fightTimeMilliseconds{}, money{}; };
struct EnemyFactionRelation { std::int32_t firstFaction{}, secondFaction{}; std::uint32_t remainingMilliseconds{}; };

class CFactionWarSys {
public:
    void AddWarType(FactionWarType type);
    [[nodiscard]] std::int32_t GetDeclareMoney(std::int32_t type) const noexcept;
    [[nodiscard]] bool IsEnemyRelation(std::int32_t first, std::int32_t second) const noexcept;
    void AddEnemyRelation(std::int32_t first, std::int32_t second, std::uint32_t duration);
    bool RemoveEnemyRelation(std::int32_t first, std::int32_t second);
    [[nodiscard]] std::vector<EnemyFactionRelation> Run(std::uint32_t elapsedMilliseconds);
    [[nodiscard]] const std::deque<EnemyFactionRelation>& Relations() const noexcept { return m_Relations; }

private:
    std::map<std::int32_t, FactionWarType> m_Types;
    std::deque<EnemyFactionRelation> m_Relations;
};
