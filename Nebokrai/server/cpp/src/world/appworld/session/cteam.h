#pragma once

#include "csession.h"

#include <cstdint>
#include <string>
#include <utility>

/*
 * Исходный владелец: WorldServer/appworld/session/cteam.cpp / cteam.h.
 * Session type 1, initial delay=125, allocation values 0/1, leader и team
 * wire подтверждены PDB/raw body. Рассылка выражена через базовый session
 * state-broadcast; transport-specific SendToMapID остаётся за message-layer.
 */
class CTeam final : public CSession
{
public:
    enum class Allocation : std::int32_t { Exclusive = 0, Shared = 1 };

    CTeam(CSessionFactory& factory,
          std::uint32_t minimumPlugs,
          std::uint32_t maximumPlugs,
          std::uint32_t lifetimeMs);
    [[nodiscard]] bool IsSessionAvailable() const override;
    void AI() override;
    bool SetLeader(std::int32_t playerId);
    void SetAllocationScheme(Allocation value);
    [[nodiscard]] Allocation GetAllocationScheme() const noexcept { return m_Allocation; }
    void SetTeamName(std::string value) { m_TeamName = std::move(value); }
    void SetPassword(std::string value) { m_Password = std::move(value); }
    [[nodiscard]] std::int32_t GetLeaderID() const noexcept { return m_LeaderId; }
    [[nodiscard]] std::uint32_t GetTeamID() const noexcept { return m_TeamId; }
    bool KickPlayer(std::int32_t playerId);
    bool Serialize(std::vector<std::uint8_t>& output) const override;
    bool Unserialize(std::span<const std::uint8_t> input, std::size_t& offset) override;

private:
    std::uint32_t m_TeamId{};
    std::string m_Password;
    std::string m_TeamName;
    std::int32_t m_LeaderId{};
    Allocation m_Allocation{Allocation::Exclusive};
    std::int32_t m_DelayTicks{125};
};
