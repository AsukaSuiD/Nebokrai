#pragma once

#include "organizing.h"

#include <cstdint>
#include <map>
#include <optional>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/organizingsystem/villagewarsys.cpp/.h.
 * Schedule, ordered заявки и phase/result lifecycle подтверждены Rust/EXE.
 * Календарные события и маршрутизация в GameServer остаются внешним owner-ом.
 */
struct VillageWarSchedule {
    std::int32_t warNumber{}, warRegionId{}, villageRegionId{};
    OrganizingTime declareTime{}, startInfoTime{}, startTime{}, endInfoTime{}, endTime{}, clearPlayerTime{};
    ECityState state{ECityState::None};
    std::vector<std::int32_t> declaringFactions;
    bool everyWeek{}, awaitingResult{}, resultCompleted{};
};

struct VillageWarResult { std::int32_t warNumber{}, regionId{}, factionId{}; };

class CVillageWarSys {
public:
    bool AddSchedule(VillageWarSchedule schedule);
    [[nodiscard]] VillageWarSchedule* GetSchedule(std::int32_t warNumber) noexcept;
    [[nodiscard]] const VillageWarSchedule* GetSchedule(std::int32_t warNumber) const noexcept;
    [[nodiscard]] ECityState GetRegionState(std::int32_t regionId) const noexcept;
    [[nodiscard]] bool IsAlreadyDeclared(std::int32_t factionId) const noexcept;
    bool Declare(std::int32_t warNumber, std::int32_t factionId);
    bool SetPhase(std::int32_t warNumber, ECityState state) noexcept;
    [[nodiscard]] std::optional<VillageWarResult> Finish(std::int32_t warNumber, std::int32_t factionId);
    [[nodiscard]] const std::map<std::int32_t, VillageWarSchedule>& Schedules() const noexcept { return m_Wars; }

private:
    std::map<std::int32_t, VillageWarSchedule> m_Wars;
};
