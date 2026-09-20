#pragma once

#include "organizing.h"

#include <array>
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <string_view>

/*
 * Исходный владелец: WorldServer/appworld/organizingsystem/organizingparam.cpp/.h.
 * Формат data/FactionParam.ini и граница уровней 1..12 подтверждены ресурсом
 * и декомпилятом. Singleton и Windows timer заменены обычным owned-объектом;
 * планирование налогов принадлежит process-owner, а не конфигурации.
 */
struct OrganizingLevelParam {
    std::int32_t maximumMembers{}, masterLevel{}, experience{}, money{};
    std::string goods;
};

class COrganizingParam {
public:
    bool Load(std::string_view source);
    [[nodiscard]] const OrganizingLevelParam* GetLevel(std::int32_t level) const noexcept;
    [[nodiscard]] std::int32_t GetMaxNumberByLevel(std::int32_t level) const noexcept;

    std::int32_t uploadIconMinimumLevel{}, uploadIconIntervalMinutes{};
    std::int32_t pronounceMinimumLevel{}, leaveWordMinimumLevel{}, rightsMinimumLevel{};
    std::int32_t createUnionMinimumLevel{}, attackVillageMinimumLevel{}, attackCityMinimumLevel{};
    std::int32_t disbandMinimumMembers{}, disbandMinutes{}, maximumContributors{};
    std::string createFactionGoods;
    std::int32_t createFactionPlayerLevel{}, createFactionMoney{};
    OrganizingTime taxTime{}, rankTime{};
    std::int32_t playerRanksCount{};

private:
    std::array<std::optional<OrganizingLevelParam>, 12> m_Levels;
};
