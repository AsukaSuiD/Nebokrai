#pragma once

#include "organizing.h"

#include <array>
#include <cstdint>
#include <map>
#include <optional>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/organizingsystem/fournationwarsys.cpp/.h.
 * PDB подтверждает фонды, восемь временных фаз и состояние региона; значения
 * wire-результата синхронизируются отдельным result owner. Здесь сохранён
 * lifecycle заявки/ответов без старых static globals и Windows timers.
 */
struct FourNationFund { std::int32_t morale{}, money{}; };
struct FourNationWarSchedule {
    std::int32_t warNumber{}, regionId{};
    OrganizingTime signupStart{}, signupEnd{}, start{}, end{}, endInfo{};
    OrganizingTime enterStart{}, enterEnd{}, refreshRegion{}, clearWar{};
    ECityState state{ECityState::None};
    bool everyWeek{};
};
struct FourNationResult { std::int32_t warNumber{}, country{}, morale{}, score{}; };

class CFourNationWarSys {
public:
    bool AddSchedule(FourNationWarSchedule schedule);
    void SetFund(std::int32_t country, FourNationFund fund);
    [[nodiscard]] const FourNationFund* GetFund(std::int32_t country) const noexcept;
    [[nodiscard]] FourNationWarSchedule* GetSchedule(std::int32_t warNumber) noexcept;
    bool SetPhase(std::int32_t warNumber, ECityState state) noexcept;
    bool BeginResultCollection(std::int32_t warNumber, std::size_t expectedServers);
    [[nodiscard]] std::optional<std::vector<FourNationResult>> AddResult(FourNationResult result);

private:
    struct Pending { std::size_t expected{}; std::map<std::int32_t, FourNationResult> byCountry; };
    std::map<std::int32_t, FourNationWarSchedule> m_Schedules;
    std::map<std::int32_t, FourNationFund> m_Funds;
    std::map<std::int32_t, Pending> m_Pending;
};
