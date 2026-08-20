#include "rsplayer.h"

#include <algorithm>

namespace
{
constexpr std::string_view kBaseColumns =
    "id,name,Account,levels,occupation,sex,Country,HEAD,HELM,BODY,GLOV,BOOT,WEAPON,BACK,"
    "HEADGEAR,FROCK,WING,MANTEAU,FAIRY,HelmLevel,BodyLevel,GlovLevel,BootLevel,WeaponLevel,"
    "BackLevel,HEADGEARLevel,FROCKLevel,WINGLevel,MANTEAULevel,FAIRYLevel,Region";
}

PlayerLoadDbResult CRsPlayer::Load(IWorldDbExecutor& database, const std::int32_t playerId)
{
    const std::vector<WorldDbValue> id{static_cast<std::int64_t>(playerId)};
    return {
        database.Execute({"SELECT * FROM CSL_PLAYER_BASE WHERE id=@P1", id}),
        database.Execute({"SELECT * FROM CSL_PLAYER_ABILITY WHERE id=@P1", id}),
        database.Execute({"SELECT * FROM CSL_PLAYER_QUEST_EX WHERE PlayerID=@P1", id}),
    };
}

WorldDbResult CRsPlayer::FindIdsByAccount(IWorldDbExecutor& database, const std::string_view account)
{
    return database.Execute({"SELECT ID FROM csl_player_base WHERE Account=@P1 ORDER BY ID",
                             {std::string(account)}});
}

WorldDbResult CRsPlayer::FindByName(IWorldDbExecutor& database, const std::string_view name)
{
    return database.Execute({"SELECT * FROM CSL_PLAYER_BASE WHERE LOWER(Name)=LOWER(@P1)",
                             {std::string(name)}});
}

std::vector<WorldDbValue> CRsPlayer::BaseParameters(const PlayerBaseDbSnapshot& s)
{
    return {static_cast<std::int64_t>(s.id), s.name, s.account,
            static_cast<std::int64_t>(s.level), static_cast<std::int64_t>(s.occupation),
            static_cast<std::int64_t>(s.sex), static_cast<std::int64_t>(s.country),
            static_cast<std::int64_t>(s.head), static_cast<std::int64_t>(s.helm),
            static_cast<std::int64_t>(s.body), static_cast<std::int64_t>(s.glove),
            static_cast<std::int64_t>(s.boot), static_cast<std::int64_t>(s.weapon),
            static_cast<std::int64_t>(s.back), static_cast<std::int64_t>(s.headgear),
            static_cast<std::int64_t>(s.frock), static_cast<std::int64_t>(s.wing),
            static_cast<std::int64_t>(s.manteau), static_cast<std::int64_t>(s.fairy),
            static_cast<std::int64_t>(s.helmLevel), static_cast<std::int64_t>(s.bodyLevel),
            static_cast<std::int64_t>(s.gloveLevel), static_cast<std::int64_t>(s.bootLevel),
            static_cast<std::int64_t>(s.weaponLevel), static_cast<std::int64_t>(s.backLevel),
            static_cast<std::int64_t>(s.headgearLevel), static_cast<std::int64_t>(s.frockLevel),
            static_cast<std::int64_t>(s.wingLevel), static_cast<std::int64_t>(s.manteauLevel),
            static_cast<std::int64_t>(s.fairyLevel), static_cast<std::int64_t>(s.region)};
}

bool CRsPlayer::CreateBase(IWorldDbExecutor& database, const PlayerBaseDbSnapshot& snapshot)
{
    return database.Execute({
        "INSERT INTO CSL_PLAYER_BASE(" + std::string(kBaseColumns) + ") "
        "VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12,@P13,@P14,@P15,@P16,"
        "@P17,@P18,@P19,@P20,@P21,@P22,@P23,@P24,@P25,@P26,@P27,@P28,@P29,@P30,@P31)",
        BaseParameters(snapshot)}).success;
}

bool CRsPlayer::SaveBase(IWorldDbExecutor& database, const PlayerBaseDbSnapshot& s)
{
    auto values = BaseParameters(s);
    values.erase(values.begin() + 2); // Account исходный UPDATE не изменяет.
    const WorldDbValue id = values.front();
    values.erase(values.begin());
    values.push_back(id);
    return database.Execute({
        "UPDATE TOP (1) CSL_PLAYER_BASE SET Name=@P1,Levels=@P2,Occupation=@P3,Sex=@P4,"
        "Country=@P5,HEAD=@P6,HELM=@P7,BODY=@P8,GLOV=@P9,BOOT=@P10,WEAPON=@P11,BACK=@P12,"
        "HEADGEAR=@P13,FROCK=@P14,WING=@P15,MANTEAU=@P16,FAIRY=@P17,HelmLevel=@P18,"
        "BodyLevel=@P19,GlovLevel=@P20,BootLevel=@P21,WeaponLevel=@P22,BackLevel=@P23,"
        "HEADGEARLevel=@P24,FROCKLevel=@P25,WINGLevel=@P26,MANTEAULevel=@P27,FAIRYLevel=@P28,"
        "Region=@P29 WHERE id=@P30",
        std::move(values)}).success;
}

bool CRsPlayer::IsSafeIdentifier(const std::string_view value)
{
    return !value.empty() && std::all_of(value.begin(), value.end(), [](const unsigned char c) {
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
               (c >= '0' && c <= '9') || c == '_';
    });
}

bool CRsPlayer::SaveAbility(IWorldDbExecutor& database,
                            const std::int32_t playerId,
                            const std::vector<PlayerAbilityDbField>& fields)
{
    if (fields.empty()) return true;
    std::string assignments;
    std::vector<WorldDbValue> parameters;
    parameters.reserve(fields.size() + 1);
    for (const PlayerAbilityDbField& field : fields) {
        if (!IsSafeIdentifier(field.column) || field.column == "ID" || field.column == "id") return false;
        if (!assignments.empty()) assignments += ',';
        assignments += '[' + field.column + "]=@P" + std::to_string(parameters.size() + 1);
        parameters.push_back(field.value);
    }
    parameters.push_back(static_cast<std::int64_t>(playerId));
    return database.Execute({
        "UPDATE TOP (1) CSL_PLAYER_ABILITY SET " + assignments +
        " WHERE ID=@P" + std::to_string(parameters.size()), std::move(parameters)}).success;
}

bool CRsPlayer::SaveQuest(IWorldDbExecutor& database,
                          const std::int32_t playerId,
                          std::vector<std::uint8_t> questData)
{
    return database.Execute({
        "IF EXISTS(SELECT 1 FROM CSL_PLAYER_QUEST_EX WHERE PlayerID=@P1) "
        "UPDATE CSL_PLAYER_QUEST_EX SET QuestData=@P2 WHERE PlayerID=@P1 "
        "ELSE INSERT INTO CSL_PLAYER_QUEST_EX(PlayerID,QuestData) VALUES(@P1,@P2)",
        {static_cast<std::int64_t>(playerId), std::move(questData)}}).success;
}

bool CRsPlayer::Restore(IWorldDbExecutor& database, const std::int32_t playerId)
{
    return database.Execute({"UPDATE csl_player_base SET DelDate=NULL WHERE ID=@P1",
                             {static_cast<std::int64_t>(playerId)}}).success;
}

bool CRsPlayer::MarkDeleted(IWorldDbExecutor& database,
                            const std::int32_t playerId,
                            std::string deletionDate)
{
    return database.Execute({"UPDATE csl_player_base SET DelDate=@P1 WHERE ID=@P2",
                             {std::move(deletionDate), static_cast<std::int64_t>(playerId)}}).success;
}
