#pragma once

#include "rssetup.h"

#include <cstdint>
#include <string>
#include <vector>

/*
 * Исходный владелец: dbaccess/worlddb/rsgodsbattle.cpp/.h.
 * Две независимые таблицы GodsBattle, их delete-before-insert и ошибочное имя
 * колонки Faciton подтверждены EXE/Rust и сохранены буквально.
 */
struct GodsBattleRegionDbRow {
    std::int32_t regionId{};
    std::int32_t factionA{};
    std::int32_t factionB{};
};
struct GodsBattleNpcDbRow { std::string npcName; std::int32_t faction{}; };

class CRsGodsBattle {
public:
    [[nodiscard]] static WorldDbResult LoadRegions(IWorldDbExecutor&);
    [[nodiscard]] static WorldDbResult LoadNpcs(IWorldDbExecutor&);
    [[nodiscard]] static bool ReplaceRegions(IWorldDbExecutor&, const std::vector<GodsBattleRegionDbRow>&);
    [[nodiscard]] static bool ReplaceNpcs(IWorldDbExecutor&, const std::vector<GodsBattleNpcDbRow>&);
    [[nodiscard]] static WorldDbResult LoadTopPlayers(IWorldDbExecutor&, std::int32_t faction);
};
