#include "savedb.h"

#include <algorithm>
#include <functional>
#include <utility>

namespace
{
bool SavePlayer(IWorldDbExecutor& database, const WorldPlayerSave& player, const bool create)
{
    if (!(create ? CRsPlayer::CreateBase(database, player.base)
                 : CRsPlayer::SaveBase(database, player.base))) return false;
    if (!CRsPlayer::SaveAbility(database, player.base.id, player.ability)) return false;
    if (!CRsPlayer::SaveQuest(database, player.base.id, player.quest)) return false;
    return CDBGoods::Replace(database, player.base.id, player.goods, player.addons);
}
}

WorldSaveReport CSaveDB::Save(IWorldDbExecutor& database, WorldSaveBatch& batch)
{
    WorldSaveReport report;
    auto phase = [&](std::string name, auto&& action) {
        ++report.phases;
        if (InTransaction(database, std::forward<decltype(action)>(action))) return true;
        report.success = false;
        report.failedPhase = std::move(name);
        return false;
    };

    if ((batch.nextPlayerId || batch.nextLeaveWordId) && !phase("идентификаторы", [&] {
            return (!batch.nextPlayerId || CRsSetup::SavePlayerId(database, *batch.nextPlayerId).success) &&
                   (!batch.nextLeaveWordId || CRsSetup::SaveLeaveWordId(database, *batch.nextLeaveWordId).success);
        })) return report;
    batch.nextPlayerId.reset();
    batch.nextLeaveWordId.reset();

    if (!batch.variables.empty() && !phase("переменные", [&] { return CRsGenVar::Save(database, batch.variables); }))
        return report;
    report.records += batch.variables.size(); batch.variables.clear();

    auto saveVector = [&](std::string phaseName, auto& values, auto&& saveOne) {
        for (auto iterator = values.begin(); iterator != values.end();) {
            if (!phase(phaseName, [&] { return saveOne(*iterator); })) return false;
            iterator = values.erase(iterator);
            ++report.records;
        }
        return true;
    };

    if (!saveVector("новый персонаж", batch.newPlayers,
                    [&](const auto& value) { return SavePlayer(database, value, true); })) return report;
    if (!saveVector("восстановление персонажа", batch.restoredPlayers,
                    [&](const auto id) { return CRsPlayer::Restore(database, id); })) return report;
    if (!saveVector("удаление персонажа", batch.deletedPlayers,
                    [&](const auto& value) { return CRsPlayer::MarkDeleted(database, value.playerId, value.deletionDate); }))
        return report;
    if (!saveVector("удаление союза", batch.deletedUnions,
                    [&](const auto id) { return CRsUnion::Delete(database, id).success; })) return report;
    if (!saveVector("удаление фракции", batch.deletedFactions,
                    [&](const auto id) { return CRsFaction::Delete(database, id).success; })) return report;
    if (!saveVector("сохранение союза", batch.unions,
                    [&](const auto& value) { return CRsUnion::Save(database, value); })) return report;
    if (!saveVector("сохранение фракции", batch.factions,
                    [&](const auto& value) { return CRsFaction::Save(database, value); })) return report;
    if (!saveVector("сохранение региона", batch.regions,
                    [&](const auto& value) { return CRsRegion::Save(database, value).success; })) return report;

    if ((!batch.godsBattleRegions.empty() || !batch.godsBattleNpcs.empty()) &&
        !phase("битва богов", [&] {
            return CRsGodsBattle::ReplaceRegions(database, batch.godsBattleRegions) &&
                   CRsGodsBattle::ReplaceNpcs(database, batch.godsBattleNpcs);
        })) return report;
    report.records += batch.godsBattleRegions.size() + batch.godsBattleNpcs.size();
    batch.godsBattleRegions.clear(); batch.godsBattleNpcs.clear();

    if (!batch.enemyFactions.empty() &&
        !phase("враждующие фракции", [&] { return CRsEnemyFactions::Replace(database, batch.enemyFactions); }))
        return report;
    report.records += batch.enemyFactions.size(); batch.enemyFactions.clear();

    if (!saveVector("сохранение страны", batch.countries,
                    [&](const auto& value) { return CDBCountry::Save(database, value).success; })) return report;
    if (!saveVector("сохранение персонажа", batch.players,
                    [&](const auto& value) { return SavePlayer(database, value, false); })) return report;
    return report;
}
