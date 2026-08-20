#include "dbcountry.h"

WorldDbResult CDBCountry::Load(IWorldDbExecutor& database)
{
    return database.Execute({"SELECT * FROM CSL_Countrys ORDER BY id", {}});
}

WorldDbResult CDBCountry::Save(IWorldDbExecutor& database, const CountrySaveSnapshot& s)
{
    WorldDbCommand command{
        "UPDATE TOP (1) CSL_Countrys SET treasury=@P1,power=@P2,tech_exp=@P3,tech_lel=@P4,"
        "king_id=@P5,king_name=@P6,king_appoint=@P7,king_salary=@P8,control_point=@P9,"
        "material_point=@P10,war_point=@P11,war_res=@P12,"
        "minister_2_id=@P13,minister_2_name=@P14,minister_2_appoint=@P15,minister_2_salary=@P16,"
        "minister_3_id=@P17,minister_3_name=@P18,minister_3_appoint=@P19,minister_3_salary=@P20,"
        "minister_4_id=@P21,minister_4_name=@P22,minister_4_appoint=@P23,minister_4_salary=@P24,"
        "minister_5_id=@P25,minister_5_name=@P26,minister_5_appoint=@P27,minister_5_salary=@P28,"
        "minister_6_id=@P29,minister_6_name=@P30,minister_6_appoint=@P31,minister_6_salary=@P32,"
        "minister_7_id=@P33,minister_7_name=@P34,minister_7_appoint=@P35,minister_7_salary=@P36 "
        "WHERE id=@P37",
        {static_cast<std::int64_t>(s.treasury), static_cast<std::int64_t>(s.power),
         static_cast<std::int64_t>(s.techExperience), static_cast<std::int64_t>(s.techLevel),
         static_cast<std::int64_t>(s.king.id), s.king.name,
         static_cast<std::int64_t>(s.king.appointed), static_cast<std::int64_t>(s.king.salaryReceived),
         static_cast<std::int64_t>(s.king.controlPoint), static_cast<std::int64_t>(s.king.materialPoint),
         static_cast<std::int64_t>(s.king.warPoint), static_cast<std::int64_t>(s.warResult)}};

    for (const auto& minister : s.ministers) {
        if (minister) {
            command.parameters.emplace_back(static_cast<std::int64_t>(minister->id));
            command.parameters.emplace_back(minister->name);
            command.parameters.emplace_back(static_cast<std::int64_t>(minister->appointed));
            command.parameters.emplace_back(static_cast<std::int64_t>(minister->salaryReceived));
        } else {
            command.parameters.emplace_back(std::int64_t{});
            command.parameters.emplace_back(std::string{});
            command.parameters.emplace_back(std::int64_t{});
            command.parameters.emplace_back(std::int64_t{});
        }
    }
    command.parameters.emplace_back(static_cast<std::uint64_t>(s.countryId));
    return database.Execute(command);
}
