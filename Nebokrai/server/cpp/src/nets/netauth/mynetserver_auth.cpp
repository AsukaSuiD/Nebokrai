#include "mynetserver_auth.h"

#include <utility>

namespace
{
constexpr std::int32_t kAuthMaxBlockConnectionsAfterHost = 10;
constexpr std::int32_t kAuthDefaultMaxInFlightSends = 100;
constexpr std::int32_t kAuthDefaultPermittedSendBytes = 0x100'0000;
}

CMyNetServerAuth::CMyNetServerAuth(asio::any_io_executor executor, std::uint32_t nowMs)
    : CServer(std::move(executor), nowMs)
{
    ConfigureSendLimits(kAuthDefaultMaxInFlightSends,
                        kAuthDefaultPermittedSendBytes);
}

void CMyNetServerAuth::ConfigureLimits(std::int32_t maxLoginServers,
                                       std::int32_t maxInFlightSends,
                                       std::int32_t permittedSendBytes,
                                       std::int32_t newAcceptTimeoutMs) noexcept
{
    ConfigureMaxClients(maxLoginServers);
    ConfigureSendLimits(maxInFlightSends, permittedSendBytes);
    ConfigureAcceptLimitsAfterHost(kAuthMaxBlockConnectionsAfterHost,
                                   newAcceptTimeoutMs);
}

std::int32_t CMyNetServerAuth::PendingMessages() const
{
    return m_ReceivedMessages.GetSize();
}

std::unique_ptr<CMessage> CMyNetServerAuth::PopReceivedMessage()
{
    return m_ReceivedMessages.PopMessage();
}

std::unique_ptr<CServerClient>
CMyNetServerAuth::CreateServerClient(std::int32_t socketId,
                                     std::uint32_t peerIPv4,
                                     std::uint32_t nowMs)
{
    return std::make_unique<CMyNetServerClientAuth>(socketId, peerIPv4, nowMs);
}

void CMyNetServerAuth::OnAccepted(CServerClient& client)
{
    static_cast<CMyNetServerClientAuth&>(client).OnAccept(m_ReceivedMessages);
}

ReceiveCallbackResult CMyNetServerAuth::OnReceive(CServerClient& client,
                                                   std::uint32_t nowMs)
{
    const AuthReceiveResult result =
        static_cast<CMyNetServerClientAuth&>(client).OnReceive(nowMs, m_ReceivedMessages);
    return {
        .success = result.Succeeded(),
        .action = ComponentReceiveErrorAction::None,
    };
}

void CMyNetServerAuth::OnClose(CServerClient& client)
{
    static_cast<CMyNetServerClientAuth&>(client).OnClose(m_ReceivedMessages);
}

void CMyNetServerAuth::OnReceiveRateExceeded(CServerClient& client,
                                              std::int32_t actual,
                                              std::int32_t permitted)
{
    static_cast<void>(client);
    static_cast<void>(actual);
    static_cast<void>(permitted);
    // Auth наследует пустой CServerClient::OnOneMessageSizeOver RVA 0x147B0.
}

void CMyNetServerAuth::OnMissingMapIDClient(std::int32_t mapId,
                                             std::int32_t socketId)
{
    static_cast<void>(mapId);
    static_cast<void>(socketId);
    // Auth-вариант не emitted producer SetClientMapID.
}

void CMyNetServerAuth::OnMissingMapNameClient(std::span<const std::uint8_t> mapName,
                                               std::int32_t socketId)
{
    static_cast<void>(mapName);
    static_cast<void>(socketId);
    // Auth-вариант не emitted producer строковой identity.
}
