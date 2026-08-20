#pragma once

#include "organizing.h"

#include <array>
#include <cstdint>
#include <deque>
#include <map>
#include <optional>
#include <set>
#include <span>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/organizingsystem/faction.cpp/.h.
 * Состояние, dirty-mask, CloneSaveData, членство и точные wire-проекции
 * подтверждены EXE/PDB и Rust-реконструкцией. std::map/set/deque заменяют
 * MSVC STL, но сохраняют сортировку ключей и порядок списков. Социальные
 * рассылки и DB-запись остаются снаружи через снимки/явные callbacks.
 */
struct FactionProperty {
    std::int32_t level{}, experience{}, offenseWins{}, defenseWins{}, villageWins{};
    std::int32_t memberCount{}, unionId{}, maximumMembers{}, upgradeExperience{};
    bool pronounce{}, leaveWords{}, endueRights{}, uploadIcon{};
    bool joinVillageWar{}, joinCityWar{}, createUnion{}, permit{};
    std::uint8_t country{};
    std::array<std::uint8_t, 3> padding{};
    std::int32_t property1{}, property2{};
};

struct FactionLeaveWord {
    std::int32_t id{}, playerId{};
    std::array<char, 20> name{};
    OrganizingTime time{};
    std::array<char, 212> content{};
};

struct FactionSaveSnapshot {
    std::int32_t id{};
    std::string name;
    std::int32_t masterId{};
    std::map<std::int32_t, OrganizingMemberInfo> members;
    FactionProperty property{};
    OrganizingTime establishedTime{};
    std::deque<std::int32_t> ownedCities;
    std::set<std::int32_t> enemyFactions;
    std::set<std::int32_t> cityWarEnemyFactions;
    std::int32_t deleteRemainTime{};
    std::int32_t changeMask{};
};

class CFaction {
public:
    explicit CFaction(std::int32_t id = 0) noexcept : m_Id(id) {}
    CFaction(std::int32_t id, std::int32_t masterId, OrganizingTime established, std::string name);

    [[nodiscard]] std::int32_t GetID() const noexcept { return m_Id; }
    [[nodiscard]] std::int32_t GetMasterID() const noexcept { return m_MasterId; }
    [[nodiscard]] const std::string& GetName() const noexcept { return m_Name; }
    [[nodiscard]] const FactionProperty& Property() const noexcept { return m_Property; }
    [[nodiscard]] FactionProperty& Property() noexcept { return m_Property; }
    [[nodiscard]] const std::map<std::int32_t, OrganizingMemberInfo>& Members() const noexcept { return m_Members; }
    [[nodiscard]] std::map<std::int32_t, OrganizingMemberInfo>& Members() noexcept { return m_Members; }
    [[nodiscard]] std::int32_t IsMember(std::int32_t playerId) const noexcept;
    [[nodiscard]] std::string GetMemberTitle(std::int32_t playerId) const;
    [[nodiscard]] bool IsContributeMember(std::int32_t playerId) const noexcept;
    bool AddMember(OrganizingMemberInfo member);
    bool RemoveMember(std::int32_t playerId);
    bool SetMemberLevel(std::int32_t playerId, std::int32_t level) noexcept;
    bool SetMemberRegion(std::int32_t playerId, std::string_view region) noexcept;
    bool SetMemberRight(std::int32_t playerId, std::size_t index, EPurviewOwnState state) noexcept;

    void AddEnemy(std::int32_t factionId) { m_EnemyFactions.insert(factionId); }
    void RemoveEnemy(std::int32_t factionId) { m_EnemyFactions.erase(factionId); }
    void AddCityWarEnemy(std::int32_t factionId) { m_CityWarEnemyFactions.insert(factionId); }
    void RemoveCityWarEnemy(std::int32_t factionId) { m_CityWarEnemyFactions.erase(factionId); }
    [[nodiscard]] bool AddOwnedCity(std::int32_t regionId);
    [[nodiscard]] bool RemoveOwnedCity(std::int32_t regionId);
    [[nodiscard]] const std::deque<std::int32_t>& OwnedCities() const noexcept { return m_OwnedCities; }

    void SetChangeData(std::int32_t mask) noexcept;
    [[nodiscard]] std::int32_t GetChangeDataType() const noexcept { return m_ChangeMask; }
    [[nodiscard]] std::optional<FactionSaveSnapshot> CloneSaveData() const;
    [[nodiscard]] bool SerializeMembers(std::vector<std::uint8_t>& output) const;
    void SetDeleteRemainTime(std::int32_t minutes) noexcept { m_DeleteRemainTime = minutes; }
    [[nodiscard]] std::int32_t GetDeleteRemainTime() const noexcept { return m_DeleteRemainTime; }
    void SetMaster(std::int32_t playerId) noexcept { m_MasterId = playerId; SetChangeData(1); }

private:
    std::int32_t m_Id{};
    std::string m_Name;
    std::int32_t m_MasterId{};
    std::map<std::int32_t, OrganizingMemberInfo> m_Members;
    FactionProperty m_Property{};
    OrganizingTime m_EstablishedTime{};
    std::deque<std::int32_t> m_OwnedCities;
    std::set<std::int32_t> m_EnemyFactions;
    std::set<std::int32_t> m_CityWarEnemyFactions;
    std::int32_t m_ChangeMask{};
    std::int32_t m_DeleteRemainTime{};
};

static_assert(sizeof(FactionProperty) == 0x38);
static_assert(sizeof(FactionLeaveWord) == 0x100);
