#include "asmessage.h"

#include <algorithm>
#include <bit>
#include <iterator>
#include <utility>

namespace Login
{
namespace
{
constexpr std::int32_t kGmaKickPlayer = 0x000CF701;
constexpr std::int32_t kGmaKickResponseToAuth = 0x000CF801;
constexpr std::int32_t kGmaKickForwardToWorld = 0x0004FD01;
constexpr std::int32_t kGmaGetServerInfo = 0x00020101;
constexpr std::int32_t kGmaGetServerInfoToAuth = 0x000CF801;
constexpr std::int32_t kGmaUpdateServerInfo = 0x00020104;
constexpr std::int32_t kGmaUpdateServerInfoToAuth = 0x000CF802;
constexpr std::int32_t kGmaBroadcastFromAuth = 0x000CF702;
constexpr std::int32_t kGmaBroadcastToWorld = 0x0004FD04;
constexpr std::int32_t kAuthConnectionClosed = 0x000CF301;
constexpr std::int32_t kLegacyReconnectedPointer = 0x000CF302;
constexpr std::int32_t kAuthResponse = 0x000CF601;
constexpr std::uint32_t kGmaAuthRangeStart = 0x000CF700U;
constexpr std::uint32_t kGmaAuthRangeEnd = 0x000CF8FFU;
constexpr std::size_t kLegacyStringLimit = 0x100U;

void AddLegacyString(CBaseMessage& message, std::span<const std::uint8_t> value)
{
    const auto terminator =
        std::find(value.begin(), value.end(), std::uint8_t{0});
    if (terminator != value.begin()) {
        message.Add(value.data(),
                    static_cast<std::int32_t>(std::distance(value.begin(), terminator)));
    }
    message.Add(std::uint8_t{0});
}

AsMessageResult Handled()
{
    return AsMessageOutcome{AsMessageHandled{}};
}
}

AsMessageHandler::AsMessageHandler(IAsMessageContext& context,
                                   AuthManager& authManager,
                                   IAuthListener& authListener) noexcept
    : m_Context(context),
      m_AuthManager(authManager),
      m_AuthListener(authListener)
{
}

AsMessageResult AsMessageHandler::OnASMessage(LoginNet::CMessage& message)
{
    const std::int32_t messageType = message.MessageType();
    const std::uint32_t opcode = std::bit_cast<std::uint32_t>(messageType);
    if (kGmaAuthRangeStart < opcode && opcode < kGmaAuthRangeEnd) {
        return OnGMAMessage(message);
    }

    switch (messageType) {
    case kAuthConnectionClosed:
        m_Context.DisconnectAuth();
        static_cast<void>(m_Context.StartReconnectThread());
        return Handled();
    case kLegacyReconnectedPointer:
        return AsMessageError{
            .kind = AsMessageErrorKind::LegacyReconnectPointerOnWire,
        };
    case kAuthResponse:
        return AsMessageOutcome{
            m_AuthManager.OnResponseAuth(message, m_AuthListener)};
    default:
        return AsMessageOutcome{AsMessageUnknown{
            .owner = UnknownAsMessageOwner::Auth,
            .messageType = messageType,
        }};
    }
}

AsMessageResult AsMessageHandler::OnGMAMessage(LoginNet::CMessage& message)
{
    switch (message.MessageType()) {
    case kGmaKickPlayer:
        return OnGMAKickPlayer(message);
    case kGmaGetServerInfo:
        message.SetMessageType(kGmaGetServerInfoToAuth);
        return RouteResult(m_Context.SendToAuth(message));
    case kGmaUpdateServerInfo:
        message.SetMessageType(kGmaUpdateServerInfoToAuth);
        message.Base().Add(m_Context.AreaID());
        return RouteResult(m_Context.SendToAuth(message));
    case kGmaBroadcastFromAuth:
        message.SetMessageType(kGmaBroadcastToWorld);
        return RouteResult(m_Context.SendAllWorld(message));
    default:
        return AsMessageOutcome{AsMessageUnknown{
            .owner = UnknownAsMessageOwner::Gma,
            .messageType = message.MessageType(),
        }};
    }
}

AsMessageResult AsMessageHandler::OnGMAKickPlayer(LoginNet::CMessage& message)
{
    const std::int32_t requestId = message.Base().GetLong();
    const auto worldName =
        message.Base().GetStrBytes(kLegacyStringLimit).value_or(
            std::vector<std::uint8_t>{});
    const char reason = message.Base().GetChar();
    const auto target =
        message.Base().GetStrBytes(kLegacyStringLimit).value_or(
            std::vector<std::uint8_t>{});
    const std::int32_t worldId = m_Context.WorldIDByName(worldName);

    if (worldId == -1) {
        LoginNet::CMessage response(kGmaKickResponseToAuth);
        response.Base().Add(requestId);
        response.Base().Add(char{0});
        AddLegacyString(response.Base(), target);

        std::vector<std::uint8_t> diagnostic{
            'L', 'o', 'g', 'i', 'n', ' ', 'S', 'e', 'r', 'v', 'e', 'r', ' ', ':', ' ',
            'I', 'n', 'v', 'a', 'l', 'i', 'd', ' ', 'w', 'o', 'r', 'l', 'd', ' ',
            's', 'e', 'r', 'v', 'e', 'r', ' ', 'n', 'a', 'm', 'e', ' ', ':', ' '};
        diagnostic.insert(diagnostic.end(), worldName.begin(), worldName.end());
        diagnostic.push_back('!');
        AddLegacyString(response.Base(), diagnostic);
        return RouteResult(m_Context.SendToAuth(response));
    }

    LoginNet::CMessage forward(kGmaKickForwardToWorld);
    forward.Base().Add(requestId);
    forward.Base().Add(reason);
    AddLegacyString(forward.Base(), target);
    return RouteResult(m_Context.SendToWorld(forward, worldId));
}

AsMessageResult AsMessageHandler::RouteResult(AsRouteResult result)
{
    if (const auto* error = std::get_if<AsRouteError>(&result)) {
        return AsMessageError{
            .kind = AsMessageErrorKind::Route,
            .routeError = *error,
        };
    }
    return Handled();
}
}
