#pragma once

#include "cplug.h"

#include <cstdint>
#include <functional>
#include <string>
#include <string_view>
#include <utility>

/*
 * Исходный владелец: WorldServer/appworld/session/cteamate.cpp/.h.
 * Plug type 5, owner region/name, state IDs 3..9 и wire suffix подтверждены
 * PDB. Проверка существования игрока отделена callback-ом от GameServer
 * transport: её минутный lifecycle сохраняется без жёсткой зависимости на
 * глобальный CGame и map sockets.
 */
class CTeamate final : public CPlug
{
public:
    enum class State : std::int32_t {
        Begin = 3, ChangeLeader = 4, ChangeAllocation = 5, ChangeRegion = 6,
        PropertiesUpdated = 7, Chat = 8, ChangeState = 9,
    };
    using ExistenceProbe = std::function<bool(std::int32_t, std::int32_t)>;

    explicit CTeamate(CSessionFactory& factory);
    void SetOwnerRegionID(std::int32_t value);
    void SetOwnerName(std::string value) { m_OwnerName = std::move(value); }
    [[nodiscard]] std::int32_t GetOwnerRegionID() const noexcept { return m_OwnerRegionId; }
    [[nodiscard]] std::string_view GetOwnerName() const noexcept { return m_OwnerName; }
    void SetExistenceProbe(ExistenceProbe probe) { m_ExistenceProbe = std::move(probe); }
    void AcceptPlayerExistenceResponse(std::int32_t regionId,
                                       std::int32_t ownerType,
                                       std::int32_t ownerId,
                                       bool exists) noexcept;
    [[nodiscard]] bool IsPlugAvailable() const override;
    bool OnChangeState(std::int32_t plugId,
                       std::int32_t state,
                       std::span<const std::uint8_t> payload) override;
    bool Serialize(std::vector<std::uint8_t>& output) const override;
    bool Unserialize(std::span<const std::uint8_t> input, std::size_t& offset) override;

private:
    std::int32_t m_OwnerRegionId{};
    std::string m_OwnerName;
    mutable std::uint32_t m_LastProbeAt{};
    mutable bool m_PlayerStillExists{true};
    ExistenceProbe m_ExistenceProbe;
};
