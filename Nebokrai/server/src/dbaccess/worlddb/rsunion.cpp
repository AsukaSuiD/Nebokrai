#include "rsunion.h"

WorldDbResult CRsUnion::Load(IWorldDbExecutor& database)
{
    return database.Execute({"SELECT * FROM CSL_UNION_BaseProperty ORDER BY ID", {}});
}

WorldDbResult CRsUnion::LoadMembers(IWorldDbExecutor& database)
{
    return database.Execute({
        "SELECT members.*, faction.Name AS FactionName "
        "FROM CSL_UNION_Members AS members "
        "JOIN CSL_FACTION_BaseProperty AS faction ON faction.ID=members.FactionID "
        "ORDER BY members.UnionID, members.FactionID", {}});
}

WorldDbResult CRsUnion::Delete(IWorldDbExecutor& database, const std::int32_t id)
{
    return database.Execute({"DELETE FROM CSL_UNION_BaseProperty WHERE ID=@P1",
                             {static_cast<std::int64_t>(id)}});
}

bool CRsUnion::Save(IWorldDbExecutor& database, const UnionSaveSnapshot& s)
{
    if (!database.Execute({
            "IF EXISTS(SELECT 1 FROM CSL_UNION_BaseProperty WHERE ID=@P1) "
            "UPDATE CSL_UNION_BaseProperty SET MasterID=@P2 WHERE ID=@P1 "
            "ELSE INSERT INTO CSL_UNION_BaseProperty(ID,Name,MasterID) VALUES(@P1,@P3,@P2)",
            {static_cast<std::int64_t>(s.id), static_cast<std::int64_t>(s.masterId), s.name}}).success)
        return false;
    if (!database.Execute({"DELETE FROM CSL_UNION_Members WHERE UnionID=@P1",
                           {static_cast<std::int64_t>(s.id)}}).success) return false;

    for (const auto& [id, member] : s.members) {
        WorldDbCommand command{
            "INSERT INTO CSL_UNION_Members"
            "(UnionID,FactionID,MemberLvl,Title,bControbute,PV_Disband,PV_Exit,PV_DubJobLvl,"
            "PV_ConMem,PV_FireOut,PV_Pronounce,PV_LeaveWord,PV_EditLeaveWord,PV_ObtainTax,"
            "PV_OperCityGate,PV_EndueROR) "
            "VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12,@P13,@P14,@P15,@P16)",
            {static_cast<std::int64_t>(s.id), static_cast<std::int64_t>(id),
             static_cast<std::int64_t>(member.level), std::string(member.Title()),
             static_cast<std::int64_t>(member.contribute)}};
        for (const EPurviewOwnState right : member.purview)
            command.parameters.emplace_back(static_cast<std::int64_t>(right));
        if (!database.Execute(command).success) return false;
    }
    return true;
}
