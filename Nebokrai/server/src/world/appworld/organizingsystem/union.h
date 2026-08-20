#pragma once

#include "organizing.h"

#include <cstdint>
#include <map>
#include <optional>
#include <string>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/organizingsystem/union.cpp/.h.
 * EXE/PDB и Rust подтверждают save-состояние, dirty-mask и одинаковую с
 * faction проверку членства. std::map сохраняет signed key-order; неподтверж-
 * дённые социальные callbacks не имитируются внутри контейнера.
 */
struct UnionSaveSnapshot {
    std::int32_t id{};
    std::string name;
    std::int32_t masterId{};
    std::map<std::int32_t, OrganizingMemberInfo> members;
    OrganizingTime establishedTime{};
    std::int32_t changeMask{};
};

class CUnion {
public:
    CUnion(std::int32_t id = 0, std::int32_t masterId = 0, std::string name = {});
    [[nodiscard]] std::int32_t GetID() const noexcept { return m_Id; }
    [[nodiscard]] std::int32_t GetMasterID() const noexcept { return m_MasterId; }
    [[nodiscard]] const std::string& GetName() const noexcept { return m_Name; }
    [[nodiscard]] std::int32_t IsMember(std::int32_t factionId) const noexcept;
    bool AddFaction(OrganizingMemberInfo faction);
    bool RemoveFaction(std::int32_t factionId);
    void SetChangeData(std::int32_t mask) noexcept;
    [[nodiscard]] std::int32_t GetChangeDataType() const noexcept { return m_ChangeMask; }
    [[nodiscard]] std::optional<UnionSaveSnapshot> CloneSaveData() const;
    [[nodiscard]] const std::map<std::int32_t, OrganizingMemberInfo>& Members() const noexcept { return m_Members; }
    [[nodiscard]] bool SerializeMembers(std::vector<std::uint8_t>& output) const;

private:
    std::int32_t m_Id{};
    std::string m_Name;
    std::int32_t m_MasterId{};
    std::map<std::int32_t, OrganizingMemberInfo> m_Members;
    OrganizingTime m_EstablishedTime{};
    std::int32_t m_ChangeMask{};
};
