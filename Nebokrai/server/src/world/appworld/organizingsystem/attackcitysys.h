#pragma once

#include "organizing.h"

#include <cstdint>
#include <map>
#include <optional>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/organizingsystem/attackcitysys.cpp/.h.
 * PDB и Rust подтверждают schedule, четыре состояния, ordered заявки и
 * раздельные фазы declare/mass/start/end/clear/refresh. Timer/network/налоги
 * не зашиваются сюда: process-owner планирует события и применяет эффекты.
 */
struct AttackCitySchedule {
    std::int32_t warNumber{}, cityRegionId{};
    OrganizingTime declareTime{}, startInfoTime{}, startTime{}, endInfoTime{}, endTime{};
    OrganizingTime massTime{}, clearPlayerTime{}, refreshRegionTime{};
    ECityState state{ECityState::None};
    std::vector<std::int32_t> declaringFactions;
    bool everyWeek{}, awaitingResult{}, resultCompleted{};
};

struct AttackCityResult { std::int32_t warNumber{}, regionId{}, factionId{}, unionId{}; };

class CAttackCitySys {
public:
    bool AddSchedule(AttackCitySchedule schedule);
    [[nodiscard]] AttackCitySchedule* GetSchedule(std::int32_t warNumber) noexcept;
    [[nodiscard]] const AttackCitySchedule* GetSchedule(std::int32_t warNumber) const noexcept;
    [[nodiscard]] ECityState GetCityState(std::int32_t regionId) const noexcept;
    [[nodiscard]] ECityState GetStateByWarNumber(std::int32_t warNumber) const noexcept;
    [[nodiscard]] bool IsAlreadyDeclared(std::int32_t factionId) const noexcept;
    bool Declare(std::int32_t warNumber, std::int32_t factionId);
    bool SetPhase(std::int32_t warNumber, ECityState state) noexcept;
    [[nodiscard]] std::optional<AttackCityResult> Finish(std::int32_t warNumber,
                                                        std::int32_t factionId,
                                                        std::int32_t unionId);
    [[nodiscard]] const std::map<std::int32_t, AttackCitySchedule>& Schedules() const noexcept { return m_Attacks; }

private:
    std::map<std::int32_t, AttackCitySchedule> m_Attacks;
};
