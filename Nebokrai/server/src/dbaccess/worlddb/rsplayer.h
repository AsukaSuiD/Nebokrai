#pragma once

#include "rssetup.h"

#include <cstdint>
#include <string>
#include <vector>

/*
 * Исходный владелец: dbaccess/worlddb/rsplayer.cpp/.h.
 * Нормализованные CSL_PLAYER_BASE/ABILITY/QUEST_EX, soft-delete и порядок
 * base->ability->quest->goods подтверждены Nworldserver.exe/PDB и Rust.
 * Огромный старый ADO owner не копируется: снимки имеют только DB-поля, SQL
 * параметризован, а неизвестные ability-колонки допускаются лишь как заранее
 * проверенные identifiers от собственного schema-loader-а.
 */
struct PlayerBaseDbSnapshot {
    std::int32_t id{};
    std::string name;
    std::string account;
    std::int32_t level{};
    std::int32_t occupation{};
    std::int32_t sex{};
    std::int32_t country{};
    std::int32_t head{}, helm{}, body{}, glove{}, boot{}, weapon{}, back{};
    std::int32_t headgear{}, frock{}, wing{}, manteau{}, fairy{};
    std::int32_t helmLevel{}, bodyLevel{}, gloveLevel{}, bootLevel{}, weaponLevel{}, backLevel{};
    std::int32_t headgearLevel{}, frockLevel{}, wingLevel{}, manteauLevel{}, fairyLevel{};
    std::int32_t region{};
};

struct PlayerAbilityDbField { std::string column; WorldDbValue value; };

struct PlayerLoadDbResult {
    WorldDbResult base;
    WorldDbResult ability;
    WorldDbResult quest;
    [[nodiscard]] explicit operator bool() const noexcept
    { return base.success && ability.success && quest.success; }
};

class CRsPlayer {
public:
    [[nodiscard]] static PlayerLoadDbResult Load(IWorldDbExecutor&, std::int32_t playerId);
    [[nodiscard]] static WorldDbResult FindIdsByAccount(IWorldDbExecutor&, std::string_view account);
    [[nodiscard]] static WorldDbResult FindByName(IWorldDbExecutor&, std::string_view name);
    [[nodiscard]] static bool CreateBase(IWorldDbExecutor&, const PlayerBaseDbSnapshot&);
    [[nodiscard]] static bool SaveBase(IWorldDbExecutor&, const PlayerBaseDbSnapshot&);
    [[nodiscard]] static bool SaveAbility(IWorldDbExecutor&, std::int32_t playerId,
                                          const std::vector<PlayerAbilityDbField>&);
    [[nodiscard]] static bool SaveQuest(IWorldDbExecutor&, std::int32_t playerId,
                                        std::vector<std::uint8_t> questData);
    [[nodiscard]] static bool Restore(IWorldDbExecutor&, std::int32_t playerId);
    [[nodiscard]] static bool MarkDeleted(IWorldDbExecutor&, std::int32_t playerId,
                                          std::string deletionDate);

private:
    [[nodiscard]] static bool IsSafeIdentifier(std::string_view);
    [[nodiscard]] static std::vector<WorldDbValue> BaseParameters(const PlayerBaseDbSnapshot&);
};
