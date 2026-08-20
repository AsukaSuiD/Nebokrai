#pragma once

#include "../dbaccess/worlddb/dbcountry.h"
#include "../dbaccess/worlddb/dbgoods.h"
#include "../dbaccess/worlddb/rsenemyfactions.h"
#include "../dbaccess/worlddb/rsfaction.h"
#include "../dbaccess/worlddb/rsgenvar.h"
#include "../dbaccess/worlddb/rsgodsbattle.h"
#include "../dbaccess/worlddb/rsplayer.h"
#include "../dbaccess/worlddb/rsregion.h"
#include "../dbaccess/worlddb/rsunion.h"

#include <cstdint>
#include <optional>
#include <string>
#include <vector>

/*
 * Исходный владелец: WorldServer/worldserver/savedb.cpp.
 * Сохраняется исходный порядок крупных фаз: setup/variables, lifecycle
 * персонажей, organization, regions, battle/enemy/country и обычные players.
 * Старый глобальный scratch-state заменён владеющим снимком. Каждый элемент
 * удаляется из очереди только после успешной транзакции, поэтому ошибка Linux
 * ODBC не теряет данные — это осознанное безопасное отличие от старых участков,
 * которые иногда очищали list даже после ADO-ошибки.
 */
struct WorldPlayerSave {
    PlayerBaseDbSnapshot base;
    std::vector<PlayerAbilityDbField> ability;
    std::vector<std::uint8_t> quest;
    std::vector<GoodsDbRow> goods;
    std::vector<GoodsAddonDbRow> addons;
};

struct WorldDeletedPlayer { std::int32_t playerId{}; std::string deletionDate; };

struct WorldSaveBatch {
    std::optional<std::int32_t> nextPlayerId;
    std::optional<std::int32_t> nextLeaveWordId;
    std::vector<GeneralVariableDbRow> variables;
    std::vector<WorldPlayerSave> newPlayers;
    std::vector<std::int32_t> restoredPlayers;
    std::vector<WorldDeletedPlayer> deletedPlayers;
    std::vector<std::int32_t> deletedUnions;
    std::vector<std::int32_t> deletedFactions;
    std::vector<UnionSaveSnapshot> unions;
    std::vector<FactionSaveSnapshot> factions;
    std::vector<RegionDbSnapshot> regions;
    std::vector<GodsBattleRegionDbRow> godsBattleRegions;
    std::vector<GodsBattleNpcDbRow> godsBattleNpcs;
    std::vector<EnemyFactionDbRow> enemyFactions;
    std::vector<CountrySaveSnapshot> countries;
    std::vector<WorldPlayerSave> players;
};

struct WorldSaveReport {
    bool success{true};
    std::size_t phases{};
    std::size_t records{};
    std::string failedPhase;
    std::string databaseError;
};

class CSaveDB {
public:
    [[nodiscard]] static WorldSaveReport Save(IWorldDbExecutor&, WorldSaveBatch&);

private:
    template<class Action>
    static bool InTransaction(IWorldDbExecutor& database, Action&& action)
    {
        if (!database.BeginTransaction()) return false;
        if (!action()) {
            static_cast<void>(database.RollbackTransaction());
            return false;
        }
        if (!database.CommitTransaction()) {
            static_cast<void>(database.RollbackTransaction());
            return false;
        }
        return true;
    }
};
