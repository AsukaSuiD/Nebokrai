#include "rsfaction.h"

#include <string>

namespace
{
std::string DbTime(const OrganizingTime& time)
{
    return std::to_string(time.year) + '-' + std::to_string(time.month) + '-' +
           std::to_string(time.day) + ' ' + std::to_string(time.hour) + ':' +
           std::to_string(time.minute) + ':' + std::to_string(time.second);
}
}

WorldDbResult CRsFaction::Load(IWorldDbExecutor& database)
{
    return database.Execute({"SELECT * FROM CSL_FACTION_BaseProperty ORDER BY ID", {}});
}

WorldDbResult CRsFaction::LoadMembers(IWorldDbExecutor& database)
{
    return database.Execute({
        "SELECT members.*, players.Name, players.Levels, players.Occupation "
        "FROM CSL_FACTION_Members AS members "
        "JOIN CSL_PLAYER_BASE AS players ON players.ID=members.PlayerID "
        "ORDER BY members.FactionID, members.PlayerID", {}});
}

WorldDbResult CRsFaction::Delete(IWorldDbExecutor& database, const std::int32_t id)
{
    return database.Execute({"DELETE FROM CSL_FACTION_BaseProperty WHERE ID=@P1",
                             {static_cast<std::int64_t>(id)}});
}

bool CRsFaction::Save(IWorldDbExecutor& database, const FactionSaveSnapshot& s)
{
    const auto& p = s.property;
    if (!database.Execute({
            "IF EXISTS(SELECT 1 FROM CSL_FACTION_BaseProperty WHERE ID=@P1) "
            "UPDATE CSL_FACTION_BaseProperty SET MasterID=@P2,Levels=@P3,Experience=@P4,"
            "OffenseVictorCounts=@P5,DefenceVictorCounts=@P6,VillageWarVictorCounts=@P7,"
            "MemberNums=@P8,UnionID=@P9,bPermit=@P10,lPro1=@P11,lPro2=@P12,"
            "DelRemainTime=@P13,country=@P14 WHERE ID=@P1 "
            "ELSE INSERT INTO CSL_FACTION_BaseProperty"
            "(ID,Name,MasterID,Levels,Experience,OffenseVictorCounts,DefenceVictorCounts,"
            "VillageWarVictorCounts,MemberNums,UnionID,bPermit,lPro1,lPro2,DelRemainTime,country) "
            "VALUES(@P1,@P15,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12,@P13,@P14)",
            {static_cast<std::int64_t>(s.id), static_cast<std::int64_t>(s.masterId),
             static_cast<std::int64_t>(p.level), static_cast<std::int64_t>(p.experience),
             static_cast<std::int64_t>(p.offenseWins), static_cast<std::int64_t>(p.defenseWins),
             static_cast<std::int64_t>(p.villageWins), static_cast<std::int64_t>(s.members.size()),
             static_cast<std::int64_t>(p.unionId), static_cast<std::int64_t>(p.permit),
             static_cast<std::int64_t>(p.property1), static_cast<std::int64_t>(p.property2),
             static_cast<std::int64_t>(s.deleteRemainTime), static_cast<std::int64_t>(p.country),
             s.name}}).success) return false;

    if (!database.Execute({"DELETE FROM CSL_FACTION_Members WHERE FactionID=@P1",
                           {static_cast<std::int64_t>(s.id)}}).success) return false;
    for (const auto& [id, member] : s.members) {
        WorldDbCommand command{
            "INSERT INTO CSL_FACTION_Members"
            "(FactionID,PlayerID,MemberLvl,Title,bControbute,LastOnlineTime,PV_Disband,PV_Exit,"
            "PV_DubJobLvl,PV_ConMem,PV_FireOut,PV_Pronounce,PV_LeaveWord,PV_EditLeaveWord,"
            "PV_ObtainTax,PV_OperCityGate,PV_EndueROR) "
            "VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12,@P13,@P14,@P15,@P16,@P17)",
            {static_cast<std::int64_t>(s.id), static_cast<std::int64_t>(id),
             static_cast<std::int64_t>(member.level), std::string(member.Title()),
             static_cast<std::int64_t>(member.contribute), DbTime(member.lastOnlineTime)}};
        for (const EPurviewOwnState right : member.purview)
            command.parameters.emplace_back(static_cast<std::int64_t>(right));
        if (!database.Execute(command).success) return false;
    }
    return true;
}
