#include "serverforgs.h"

#include <utility>

namespace BillingNet
{
namespace
{
constexpr std::int32_t kDefaultMaxInFlightSends = 100;
constexpr std::int32_t kDefaultPermittedSendBytes = 0x100'0000;
}

CServerForGS::CServerForGS(asio::any_io_executor executor, std::uint32_t nowMs)
    : CServer(std::move(executor), nowMs)
{
    ConfigureSendLimits(kDefaultMaxInFlightSends, kDefaultPermittedSendBytes);
}

std::int32_t CServerForGS::PendingMessages() const
{
    return m_ReceivedMessages.GetSize();
}

std::unique_ptr<CMessage> CServerForGS::PopReceivedMessage()
{
    return m_ReceivedMessages.PopMessage();
}

std::unique_ptr<CServerClient>
CServerForGS::CreateServerClient(std::int32_t socketId,
                                 std::uint32_t peerIPv4,
                                 std::uint32_t nowMs)
{
    return std::make_unique<CClientForGS>(socketId, peerIPv4, nowMs);
}

void CServerForGS::OnAccepted(CServerClient& client)
{
    client.MarkOpen();
}

ReceiveCallbackResult CServerForGS::OnReceive(CServerClient& client,
                                               std::uint32_t nowMs)
{
    static_cast<void>(nowMs);
    const BillingReceiveResult result =
        static_cast<CClientForGS&>(client).OnReceive(m_ReceivedMessages);
    return {static_cast<bool>(result), ComponentReceiveErrorAction::None};
}

void CServerForGS::OnClose(CServerClient& client)
{
    static_cast<CClientForGS&>(client).OnClose(m_ReceivedMessages);
}

void CServerForGS::OnReceiveRateExceeded(CServerClient& client,
                                          std::int32_t actual,
                                          std::int32_t permitted)
{
    static_cast<void>(client);
    static_cast<void>(actual);
    static_cast<void>(permitted);
}

void CServerForGS::OnMissingMapIDClient(std::int32_t mapId,
                                         std::int32_t socketId)
{
    static_cast<void>(mapId);
    static_cast<void>(socketId);
}

void CServerForGS::OnMissingMapNameClient(std::span<const std::uint8_t> mapName,
                                           std::int32_t socketId)
{
    static_cast<void>(mapName);
    static_cast<void>(socketId);
}
}
