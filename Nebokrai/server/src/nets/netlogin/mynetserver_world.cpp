#include "mynetserver_world.h"

#include <bit>
#include <utility>

namespace LoginNet
{
namespace
{
constexpr std::int32_t kDefaultMaxInFlightSends = 100;
constexpr std::int32_t kDefaultPermittedSendBytes = 0x100'0000;
}

CMyNetServerWorld::CMyNetServerWorld(asio::any_io_executor executor,
                                     std::uint32_t nowMs)
    : CServer(std::move(executor), nowMs)
{
    ConfigureSendLimits(kDefaultMaxInFlightSends, kDefaultPermittedSendBytes);
}

void CMyNetServerWorld::ConfigureReceive(bool enabled,
                                         std::uint32_t maximumBytesPerSecond,
                                         std::uint32_t forbidTimeMs) noexcept
{
    ConfigureReceiveGuard(enabled,
                          std::bit_cast<std::int32_t>(maximumBytesPerSecond),
                          std::bit_cast<std::int32_t>(forbidTimeMs));
}

void CMyNetServerWorld::ConfigureConnectionLimits(std::int32_t maximumClients,
                                                   std::int32_t maximumInFlightSends,
                                                   std::int32_t permittedSendBytes) noexcept
{
    ConfigureMaxClients(maximumClients);
    ConfigureSendLimits(maximumInFlightSends, permittedSendBytes);
}

std::int32_t CMyNetServerWorld::PendingMessages() const
{
    return m_ReceivedMessages.GetSize();
}

std::unique_ptr<CMessage> CMyNetServerWorld::PopReceivedMessage()
{
    return m_ReceivedMessages.PopMessage();
}

std::unique_ptr<CServerClient>
CMyNetServerWorld::CreateServerClient(std::int32_t socketId,
                                      std::uint32_t peerIPv4,
                                      std::uint32_t nowMs)
{
    return std::make_unique<CMyNetServerClientWorld>(socketId, peerIPv4, nowMs);
}

void CMyNetServerWorld::OnAccepted(CServerClient& client)
{
    client.MarkOpen();
}

ReceiveCallbackResult CMyNetServerWorld::OnReceive(CServerClient& client,
                                                    std::uint32_t nowMs)
{
    static_cast<void>(nowMs);
    const WorldReceiveResult result =
        static_cast<CMyNetServerClientWorld&>(client).OnReceive(m_ReceivedMessages);
    return {static_cast<bool>(result), ComponentReceiveErrorAction::None};
}

void CMyNetServerWorld::OnClose(CServerClient& client)
{
    static_cast<CMyNetServerClientWorld&>(client).OnClose(m_ReceivedMessages);
}

void CMyNetServerWorld::OnReceiveRateExceeded(CServerClient& client,
                                               std::int32_t actual,
                                               std::int32_t permitted)
{
    static_cast<void>(client);
    static_cast<void>(actual);
    static_cast<void>(permitted);
}

void CMyNetServerWorld::OnMissingMapIDClient(std::int32_t mapId,
                                              std::int32_t socketId)
{
    static_cast<void>(mapId);
    static_cast<void>(socketId);
}

void CMyNetServerWorld::OnMissingMapNameClient(std::span<const std::uint8_t> mapName,
                                                std::int32_t socketId)
{
    static_cast<void>(mapName);
    static_cast<void>(socketId);
}
}
