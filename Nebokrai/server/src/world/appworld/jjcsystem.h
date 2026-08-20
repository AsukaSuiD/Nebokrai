#pragma once

#include <array>
#include <cstdint>
#include <map>
#include <optional>
#include <set>
#include <string>
#include <utility>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/jjcsystem.cpp/.h. Rust/EXE
 * подтверждают queue/fighting registries, наборы регионов, rank layout,
 * weekly gate и timeout fights. DB, INI и network эффекты возвращаются как
 * действия process-owner-у; singleton и worker thread сюда не перенесены.
 */
struct JjcRank {
    std::int32_t rank{}, playerId{};
    std::uint32_t jjcLevel{};
    std::int32_t level{};
    std::array<char, 32> name{};
};
struct JjcInfo {
    std::uint32_t jjcLevel{};
    std::int32_t oldRegionId{}, positionX{}, positionY{}, opponentId{}, jjcRegionId{}, startTime{};
};
struct JjcFight { std::int32_t regionId{}, firstPlayerId{}, secondPlayerId{}, startedAt{}; };

class CJJcSystem {
public:
    struct RunResult { std::vector<JjcFight> timedOut; std::vector<std::int32_t> recycledRegions; bool weeklyReset{}; bool seasonReset{}; };

    void ConfigureWeeklyReset(std::int32_t weekDay, std::int32_t hour, std::int32_t minute, std::int32_t second) noexcept;
    void ConfigureRegions(std::int32_t first, std::int32_t last);
    void ConfigureLevelStep(std::int32_t minimum, std::int32_t maximum, std::int32_t step);
    bool Apply(std::int32_t playerId, JjcInfo info);
    bool Quit(std::int32_t playerId);
    [[nodiscard]] std::optional<std::int32_t> MatchOpponent(std::int32_t playerId) const noexcept;
    [[nodiscard]] std::optional<std::int32_t> AcquireRegion();
    bool StartFight(std::int32_t regionId, std::int32_t first, std::int32_t second, std::int32_t now);
    bool EndFight(std::int32_t regionId);
    [[nodiscard]] RunResult Run(std::int32_t now,
                                std::int32_t weekDay,
                                std::int32_t hour,
                                std::int32_t minute,
                                std::int32_t timeoutSeconds);
    void SetRanks(std::vector<JjcRank> ranks) { m_Ranks = std::move(ranks); }
    [[nodiscard]] const std::vector<JjcRank>& Ranks() const noexcept { return m_Ranks; }

private:
    [[nodiscard]] std::int32_t LevelStep(std::uint32_t jjcLevel) const noexcept;
    std::map<std::int32_t, JjcInfo> m_Queue;
    std::map<std::int32_t, JjcFight> m_Fights;
    std::set<std::int32_t> m_AvailableRegions, m_UsedRegions;
    std::map<std::pair<std::int32_t,std::int32_t>,std::int32_t> m_LevelSteps;
    std::vector<JjcRank> m_Ranks;
    std::int32_t m_ResetDay{}, m_ResetHour{}, m_ResetMinute{}, m_ResetSecond{};
    std::int32_t m_PassedWeeks{};
    bool m_UpdatedThisWeek{};
};
