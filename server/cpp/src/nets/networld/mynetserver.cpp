#include "mynetserver.h"

#include <utility>

namespace WorldNet
{
namespace
{
constexpr std::int32_t kDefaultMaxInFlightSends = 100;
constexpr std::int32_t kDefaultPermittedSendBytes = 0x200'0000;
}

CMyNetServer::CMyNetServer(asio::any_io_executor executor, std::uint32_t nowMs)
    : CServer(std::move(executor), nowMs)
{
    ConfigureSendLimits(kDefaultMaxInFlightSends, kDefaultPermittedSendBytes);
}

std::int32_t CMyNetServer::PendingMessages() const
{
    return m_ReceivedMessages.GetSize();
}

std::unique_ptr<CMessage> CMyNetServer::PopReceivedMessage()
{
    return m_ReceivedMessages.PopMessage();
}

std::unique_ptr<CServerClient>
CMyNetServer::CreateServerClient(std::int32_t socketId,
                                 std::uint32_t peerIPv4,
                                 std::uint32_t nowMs)
{
    return std::make_unique<CMyServerClient>(socketId, peerIPv4, nowMs);
}

void CMyNetServer::OnAccepted(CServerClient& client) { client.MarkOpen(); }

ReceiveCallbackResult CMyNetServer::OnReceive(CServerClient& client,
                                              std::uint32_t nowMs)
{
    const GameServerReceiveResult result =
        static_cast<CMyServerClient&>(client).OnReceive(m_ReceivedMessages, nowMs);
    return {static_cast<bool>(result), ComponentReceiveErrorAction::None};
}

void CMyNetServer::OnClose(CServerClient& client)
{
    static_cast<CMyServerClient&>(client).OnClose(m_ReceivedMessages);
}

void CMyNetServer::OnReceiveRateExceeded(CServerClient& client,
                                         std::int32_t actual,
                                         std::int32_t permitted)
{
    static_cast<void>(client);
    static_cast<void>(actual);
    static_cast<void>(permitted);
}

void CMyNetServer::OnMissingMapIDClient(std::int32_t mapId, std::int32_t socketId)
{
    static_cast<void>(mapId);
    static_cast<void>(socketId);
}

void CMyNetServer::OnMissingMapNameClient(std::span<const std::uint8_t> mapName,
                                          std::int32_t socketId)
{
    static_cast<void>(mapName);
    static_cast<void>(socketId);
}
}
