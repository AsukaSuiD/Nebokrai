#include "mynetserver_client.h"

#include <bit>
#include <utility>

namespace LoginNet
{
namespace
{
constexpr std::int32_t kDefaultMaxInFlightSends = 5;
constexpr std::int32_t kDefaultPermittedSendBytes = 0x40'0000;
constexpr std::int32_t kClientIdentityError = 0x00010001;
}

CMyNetServerClient::CMyNetServerClient(asio::any_io_executor executor,
                                       std::uint32_t nowMs)
    : CServer(std::move(executor), nowMs)
{
    ConfigureSendLimits(kDefaultMaxInFlightSends, kDefaultPermittedSendBytes);
}

void CMyNetServerClient::ConfigureReceive(bool checkLengthCrc,
                                           bool checkContentCrc,
                                           std::uint32_t maximumBytesPerSecond,
                                           std::uint32_t forbidTimeMs,
                                           std::uint32_t maximumMessageLength) noexcept
{
    ConfigureReceiveGuard(checkLengthCrc,
                          std::bit_cast<std::int32_t>(maximumBytesPerSecond),
                          std::bit_cast<std::int32_t>(forbidTimeMs));
    m_ReceiveSettings = {
        checkLengthCrc,
        checkContentCrc,
        maximumMessageLength,
    };
}

void CMyNetServerClient::ConfigureConnectionLimits(std::int32_t maximumClients,
                                                    std::int32_t maximumInFlightSends,
                                                    std::int32_t permittedSendBytes) noexcept
{
    ConfigureMaxClients(maximumClients);
    ConfigureSendLimits(maximumInFlightSends, permittedSendBytes);
}

std::int32_t CMyNetServerClient::PendingMessages() const
{
    return m_ReceivedMessages.GetSize();
}

std::unique_ptr<CMessage> CMyNetServerClient::PopReceivedMessage()
{
    return m_ReceivedMessages.PopMessage();
}

std::optional<ClientNetworkNotice> CMyNetServerClient::PopNotice()
{
    if (m_Notices.empty()) {
        return std::nullopt;
    }
    ClientNetworkNotice notice = std::move(m_Notices.front());
    m_Notices.pop_front();
    return notice;
}

std::unique_ptr<CServerClient>
CMyNetServerClient::CreateServerClient(std::int32_t socketId,
                                       std::uint32_t peerIPv4,
                                       std::uint32_t nowMs)
{
    return std::make_unique<CMyNetServerClientClient>(socketId, peerIPv4, nowMs);
}

void CMyNetServerClient::OnAccepted(CServerClient& client)
{
    client.MarkOpen();
}

ReceiveCallbackResult CMyNetServerClient::OnReceive(CServerClient& client,
                                                     std::uint32_t nowMs)
{
    static_cast<void>(nowMs);
    auto& concrete = static_cast<CMyNetServerClientClient&>(client);
    const ClientReceiveResult result = concrete.OnReceive(m_ReceiveSettings, m_ReceivedMessages);
    if (!result.error) {
        return {true, ComponentReceiveErrorAction::None};
    }

    if (result.error->kind == ClientReceiveErrorKind::MessageLengthExceeded) {
        const auto context = client.MessageContext();
        m_Notices.emplace_back(ClientOneMessageSizeExceeded{
            context.socketId,
            {context.mapName.begin(), context.mapName.end()},
            context.peerIPv4,
            std::bit_cast<std::int32_t>(result.error->declared),
            std::bit_cast<std::int32_t>(result.error->permitted),
        });
    }

    return {
        false,
        result.error->RequiresForbidAndQuit()
            ? ComponentReceiveErrorAction::ForbidAndQuit
            : ComponentReceiveErrorAction::None,
    };
}

void CMyNetServerClient::OnClose(CServerClient& client)
{
    static_cast<void>(
        static_cast<CMyNetServerClientClient&>(client).OnClose(m_ReceivedMessages));
}

void CMyNetServerClient::OnReceiveRateExceeded(CServerClient& client,
                                                std::int32_t actual,
                                                std::int32_t permitted)
{
    const auto context = client.MessageContext();
    m_Notices.emplace_back(ClientTotalMessageSizeExceeded{
        context.socketId,
        {context.mapName.begin(), context.mapName.end()},
        context.peerIPv4,
        actual,
        permitted,
    });
}

void CMyNetServerClient::OnMissingMapIDClient(std::int32_t mapId,
                                               std::int32_t socketId)
{
    static_cast<void>(mapId);
    static_cast<void>(socketId);
}

void CMyNetServerClient::OnMissingMapNameClient(std::span<const std::uint8_t> mapName,
                                                 std::int32_t socketId)
{
    static_cast<void>(socketId);
    auto message = std::make_unique<CMessage>(kClientIdentityError);
    if (!mapName.empty()) {
        message->Base().Add(mapName.data(), static_cast<std::int32_t>(mapName.size()));
    }
    message->Base().Add(std::uint8_t{0});
    static_cast<void>(m_ReceivedMessages.PushMessage(std::move(message)));
}
}
