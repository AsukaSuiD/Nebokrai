#include "message_func.h"

#include <algorithm>
#include <bit>
#include <cstdio>
#include <string>
#include <utility>

namespace
{
std::uint8_t LowerWindows1251Byte(std::uint8_t byte) noexcept
{
    if (byte >= 'A' && byte <= 'Z') {
        return static_cast<std::uint8_t>(byte + ('a' - 'A'));
    }
    if (byte >= 0xC0U && byte <= 0xDFU) {
        return static_cast<std::uint8_t>(byte + 0x20U);
    }

    // Однобайтовые пары Windows-1251, которые Unicode lowercase меняет помимо
    // ASCII и русского диапазона. Undefined/неоднобайтовые значения остаются
    // буквальными, как в сохранённой поздней реконструкции.
    switch (byte) {
    case 0x80: return 0x90; // Ђ -> ђ
    case 0x81: return 0x83; // Ѓ -> ѓ
    case 0x8A: return 0x9A; // Љ -> љ
    case 0x8C: return 0x9C; // Њ -> њ
    case 0x8D: return 0x9D; // Ќ -> ќ
    case 0x8E: return 0x9E; // Ћ -> ћ
    case 0x8F: return 0x9F; // Џ -> џ
    case 0xA1: return 0xA2; // Ў -> ў
    case 0xA3: return 0xBC; // Ј -> ј
    case 0xA5: return 0xB4; // Ґ -> ґ
    case 0xA8: return 0xB8; // Ё -> ё
    case 0xAA: return 0xBA; // Є -> є
    case 0xAF: return 0xBF; // Ї -> ї
    case 0xB2: return 0xB3; // І -> і
    case 0xBD: return 0xBE; // Ѕ -> ѕ
    default: return byte;
    }
}

std::vector<std::uint8_t> LegacyLowercase(std::vector<std::uint8_t> value)
{
    std::transform(value.begin(), value.end(), value.begin(), LowerWindows1251Byte);
    return value;
}

void AddLegacyString(CBaseMessage& message, std::span<const std::uint8_t> value)
{
    const auto terminator = std::find(value.begin(), value.end(), std::uint8_t{0});
    const std::size_t length = static_cast<std::size_t>(std::distance(value.begin(), terminator));
    if (length != 0U) {
        message.Add(value.data(), static_cast<std::int32_t>(length));
    }
    message.Add(std::uint8_t{0});
}

IPv4Octets PeerAddress(const CMessage& message) noexcept
{
    const std::uint32_t ip = message.IP();
    return {
        static_cast<std::uint8_t>(ip),
        static_cast<std::uint8_t>(ip >> 8U),
        static_cast<std::uint8_t>(ip >> 16U),
        static_cast<std::uint8_t>(ip >> 24U),
    };
}

AuthDispatchStatus SendMessage(const CMessage& message,
                               const ServerCommandHandle& sender,
                               std::int32_t socketId)
{
    const auto result = message.SendToLogin(sender, socketId);
    if (std::holds_alternative<AuthSendMessageError>(result)) {
        return AuthDispatchStatus::OutgoingMessageTooLarge;
    }
    return AuthDispatchStatus::Ok;
}
}

AuthMessageHandlers::AuthMessageHandlers(bool ipFilterEnabled,
                                         std::vector<kl_net::IpPattern> allowedPatterns,
                                         DbQuestSink pushQuest,
                                         ServerInfoSink pushServerInfo)
    : m_IpFilterEnabled(ipFilterEnabled),
      m_PushQuest(std::move(pushQuest)),
      m_PushServerInfo(std::move(pushServerInfo))
{
    m_IpAllower.ReplacePatterns(std::move(allowedPatterns));
}

void AuthMessageHandlers::ReplaceIpFilter(bool enabled,
                                          std::vector<kl_net::IpPattern> allowedPatterns)
{
    m_IpFilterEnabled = enabled;
    m_IpAllower.ReplacePatterns(std::move(allowedPatterns));
}

std::int32_t AuthMessageHandlers::LoginServerSocketID(std::int32_t areaId) const noexcept
{
    const auto found = m_LoginServers.find(areaId);
    return found == m_LoginServers.end() ? 0 : found->second;
}

std::int32_t AuthMessageHandlers::LoginServerAreaID(std::int32_t socketId) const noexcept
{
    return AreaForSocket(socketId).value_or(0);
}

std::vector<std::int32_t> AuthMessageHandlers::LoginServerSocketIDs() const
{
    std::vector<std::int32_t> result;
    result.reserve(m_LoginServers.size());
    for (const auto& [_, socketId] : m_LoginServers) {
        result.push_back(socketId);
    }
    return result;
}

std::optional<LoginServerNotice> AuthMessageHandlers::PopNotice()
{
    if (m_Notices.empty()) {
        return std::nullopt;
    }
    LoginServerNotice notice = std::move(m_Notices.front());
    m_Notices.pop_front();
    return notice;
}

AuthDispatchStatus AuthMessageHandlers::Handle(AuthMessageKind kind,
                                               CMessage& message,
                                               const ServerCommandHandle& sender)
{
    switch (kind) {
    case AuthMessageKind::LoginServerConnected:
        OnLoginServerConnect(message, sender);
        return AuthDispatchStatus::Ok;
    case AuthMessageKind::LoginServerDisconnected:
        OnLoginServerDisconnect(message);
        return AuthDispatchStatus::Ok;
    case AuthMessageKind::AuthenticateAccount:
        OnAuthAccount(message, false);
        return AuthDispatchStatus::Ok;
    case AuthMessageKind::AuthenticateAccountExtended:
        OnAuthAccount(message, true);
        return AuthDispatchStatus::Ok;
    case AuthMessageKind::GetLoginServerInfo:
        OnLoginServerGetInfo(message);
        return AuthDispatchStatus::Ok;
    case AuthMessageKind::UpdateServerInfoResponse:
        OnUpdateServerInfoResponse(message);
        return AuthDispatchStatus::Ok;
    case AuthMessageKind::GmKickPlayer:
        return OnGmKickPlayer(message, sender);
    case AuthMessageKind::KickPlayerResponse:
        return OnKickPlayerResponse(message, sender);
    case AuthMessageKind::GmLockAccount:
        OnGmLockAccount(message);
        return AuthDispatchStatus::Ok;
    }
    return AuthDispatchStatus::Ok;
}

void AuthMessageHandlers::OnLoginServerConnect(const CMessage& message,
                                                const ServerCommandHandle& sender)
{
    const IPv4Octets address = PeerAddress(message);
    const bool allowed = !m_IpFilterEnabled || m_IpAllower.IsAllowed(address);
    m_Notices.emplace_back(LoginServerConnectedNotice{address, message.SocketID(), allowed});
    if (!allowed) {
        static_cast<void>(sender.QuitBySocketID(message.SocketID()));
    }
}

void AuthMessageHandlers::OnLoginServerDisconnect(const CMessage& message)
{
    const std::int32_t socketId = message.SocketID();
    const std::int32_t areaId = AreaForSocket(socketId).value_or(0);
    m_Notices.emplace_back(
        LoginServerDisconnectedNotice{PeerAddress(message), areaId, socketId});

    if (const auto foundArea = AreaForSocket(socketId)) {
        m_LoginServers.erase(*foundArea);
    }
}

void AuthMessageHandlers::OnLoginServerGetInfo(CMessage& message)
{
    const std::int32_t areaId = message.Base().GetLong();
    const std::int32_t socketId = message.SocketID();
    m_Notices.emplace_back(LoginServerRegisteredNotice{PeerAddress(message), areaId, socketId});
    m_LoginServers[areaId] = socketId;
}

void AuthMessageHandlers::OnUpdateServerInfoResponse(CMessage& message)
{
    AuthDb::ServerInfo info{
        .playerCount = message.Base().GetLong(),
        .gameServerId = message.Base().GetLong(),
        .worldServerId = message.Base().GetLong(),
        .loginServerId = message.Base().GetLong(),
    };
    if (m_PushServerInfo) {
        m_PushServerInfo(std::move(info));
    }
}

void AuthMessageHandlers::OnAuthAccount(CMessage& message, bool extended)
{
    AuthDb::AuthQuestData request{
        .account = LegacyLowercase(message.GetStr()),
        .password = LegacyLowercase(message.GetStr()),
        .clientIp = std::bit_cast<std::uint32_t>(message.Base().GetLong()),
        .clientSocketId = message.Base().GetLong(),
    };

    if (!m_PushQuest) {
        return;
    }
    if (extended) {
        static_cast<void>(m_PushQuest(AuthDb::AuthenticateExtendedQuest{
            message.SocketID(), std::move(request)}));
    } else {
        static_cast<void>(m_PushQuest(AuthDb::AuthenticateQuest{
            message.SocketID(), std::move(request)}));
    }
}

void AuthMessageHandlers::OnGmLockAccount(CMessage& message)
{
    AuthDb::LockQuestData request;
    request.account = message.GetStr();
    request.until = {
        .year = message.Base().GetWord(),
        .month = message.Base().GetWord(),
        .day = message.Base().GetWord(),
        .hour = message.Base().GetWord(),
        .minute = message.Base().GetWord(),
        .second = message.Base().GetWord(),
    };
    if (m_PushQuest) {
        static_cast<void>(m_PushQuest(AuthDb::LockQuest{
            message.SocketID(), std::move(request)}));
    }
}

AuthDispatchStatus AuthMessageHandlers::OnGmKickPlayer(
    CMessage& message,
    const ServerCommandHandle& sender) const
{
    const std::int32_t areaId = message.Base().GetLong();
    const std::vector<std::uint8_t> account = message.GetStr();
    const char reason = message.Base().GetChar();
    const std::vector<std::uint8_t> operatorName = message.GetStr();
    const std::int32_t targetSocketId = LoginServerSocketID(areaId);

    if (targetSocketId == 0) {
        CMessage response(0x0010F201);
        response.Base().Add(char{0});
        AddLegacyString(response.Base(), operatorName);

        char detail[128]{};
        std::snprintf(detail,
                      sizeof(detail),
                      "AuthServer : Invalid login server area id : %d!",
                      areaId);
        AddLegacyString(response.Base(), std::span<const std::uint8_t>(
                                             reinterpret_cast<const std::uint8_t*>(detail),
                                             std::char_traits<char>::length(detail)));
        return SendMessage(response, sender, message.SocketID());
    }

    CMessage forward(0x000CF701);
    forward.Base().Add(message.SocketID());
    AddLegacyString(forward.Base(), account);
    forward.Base().Add(reason);
    AddLegacyString(forward.Base(), operatorName);
    return SendMessage(forward, sender, targetSocketId);
}

AuthDispatchStatus AuthMessageHandlers::OnKickPlayerResponse(
    CMessage& message,
    const ServerCommandHandle& sender) const
{
    const std::int32_t returnSocketId = message.Base().GetLong();
    const char result = message.Base().GetChar();

    CMessage response(0x0010F201);
    response.Base().Add(result);
    if (result == 0) {
        const auto account = message.GetStr();
        const auto detail = message.GetStr();
        AddLegacyString(response.Base(), account);
        AddLegacyString(response.Base(), detail);
    } else {
        const auto detail = message.GetStr();
        AddLegacyString(response.Base(), detail);
    }
    return SendMessage(response, sender, returnSocketId);
}

std::optional<std::int32_t>
AuthMessageHandlers::AreaForSocket(std::int32_t socketId) const noexcept
{
    for (const auto& [areaId, registeredSocket] : m_LoginServers) {
        if (registeredSocket == socketId) {
            return areaId;
        }
    }
    return std::nullopt;
}
