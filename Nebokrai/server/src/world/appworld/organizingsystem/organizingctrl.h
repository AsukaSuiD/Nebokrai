#pragma once

#include "faction.h"
#include "union.h"

#include <chrono>
#include <cstdint>
#include <deque>
#include <functional>
#include <map>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/organizingsystem/organizingctrl.cpp/.h.
 * EXE/PDB и Rust подтверждают sorted membership lookup, save/delete очереди,
 * countdown распуска faction и top-info lifetime. Singleton заменён owned
 * controller, GetTickCount — steady_clock; внешние DB/network эффекты выражены
 * снимками и callbacks, поэтому этот owner не знает транспорта.
 */
struct OrganizingSaveBatch {
    std::vector<FactionSaveSnapshot> factions;
    std::vector<UnionSaveSnapshot> unions;
    std::vector<std::int32_t> deletedFactions;
    std::vector<std::int32_t> deletedUnions;
};

struct OrganizingTopInfo {
    std::int32_t id{}, timerFlag{}, parameter{};
    std::uint32_t startedAtMilliseconds{};
    std::string info;
};

class COrganizingCtrl {
public:
    using Clock = std::function<std::uint32_t()>;

    explicit COrganizingCtrl(Clock clock = {});
    bool AddFaction(std::unique_ptr<CFaction> faction);
    bool AddUnion(std::unique_ptr<CUnion> unionValue);
    [[nodiscard]] CFaction* GetFaction(std::int32_t factionId) noexcept;
    [[nodiscard]] const CFaction* GetFaction(std::int32_t factionId) const noexcept;
    [[nodiscard]] CUnion* GetUnion(std::int32_t unionId) noexcept;
    [[nodiscard]] std::int32_t FindPlayerFaction(std::int32_t playerId) const noexcept;
    [[nodiscard]] std::int32_t FindFactionUnion(std::int32_t factionId) const noexcept;
    bool DeleteFaction(std::int32_t factionId);
    bool DeleteUnion(std::int32_t unionId);
    [[nodiscard]] OrganizingSaveBatch GenerateSaveData(bool forceAll);
    std::int32_t AddTopInfo(std::int32_t timerFlag, std::int32_t parameter, std::string info);
    bool RemoveTopInfo(std::int32_t id);
    [[nodiscard]] std::vector<OrganizingTopInfo> ActiveTopInfo() const;
    std::size_t Run(std::int32_t elapsedMinutes,
                    const std::function<void(std::int32_t, std::int32_t)>& disband);

private:
    [[nodiscard]] std::uint32_t Now() const;
    std::map<std::int32_t, std::unique_ptr<CFaction>> m_Factions;
    std::map<std::int32_t, std::unique_ptr<CUnion>> m_Unions;
    std::deque<std::int32_t> m_DeletedFactions;
    std::deque<std::int32_t> m_DeletedUnions;
    std::deque<OrganizingTopInfo> m_TopInfo;
    Clock m_Clock;
    std::int32_t m_NextTopInfoId{1};
};
