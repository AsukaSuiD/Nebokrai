#include "servermessage.h"

#include <array>
#include <bit>
#include <ctime>
#include <string>
#include <string_view>
#include <utility>

namespace Login
{
namespace
{
constexpr std::int32_t kWorldConnectedMessageType = 0x0001FE01;
constexpr std::int32_t kWorldDisconnectedMessageType = 0x0000FF01;
constexpr std::int32_t kWorldConnectedAckMessageType = 0x0004FC03;
constexpr std::int32_t kWorldUsersSnapshotMessageType = 0x0001FE02;
constexpr std::int32_t kClearAccountsMessageType = 0x0001FE03;
constexpr std::int32_t kWorldTelemetryMessageType = 0x0001FE04;
constexpr std::int32_t kGameServerConnectedLogMessageType = 0x0001FE05;
constexpr std::int32_t kWorldServerLogMessageType = 0x0001FE06;
constexpr std::int32_t kTypedServerLogMessageType = 0x0001FE08;
constexpr std::size_t kServerStringLimit = 0x100U;
constexpr std::size_t kGameServerIpLimit = 0xFFU;
constexpr std::size_t kServerLogTextLimit = 0x80U;

void AppendAscii(std::vector<std::uint8_t>& target, std::string_view value)
{
    target.insert(target.end(), value.begin(), value.end());
}

std::vector<std::uint8_t> FormatLocalNow(const char* format)
{
    const std::time_t now = std::time(nullptr);
    std::tm local{};
#if defined(_WIN32)
    if (::localtime_s(&local, &now) != 0) {
        return {};
    }
#else
    if (::localtime_r(&now, &local) == nullptr) {
        return {};
    }
#endif

    std::array<char, 32> buffer{};
    const std::size_t length = std::strftime(buffer.data(), buffer.size(), format, &local);
    return {reinterpret_cast<const std::uint8_t*>(buffer.data()),
            reinterpret_cast<const std::uint8_t*>(buffer.data() + length)};
}

std::vector<std::uint8_t> LegacyIPv4Text(std::uint32_t address)
{
    std::vector<std::uint8_t> text;
    for (std::uint32_t index = 0; index < 4U; ++index) {
        if (index != 0U) {
            text.push_back('.');
        }
        const std::string octet = std::to_string((address >> (index * 8U)) & 0xFFU);
        AppendAscii(text, octet);
    }
    return text;
}

std::vector<std::uint8_t> WorldConnectedDescription(
    std::span<const std::uint8_t> worldName)
{
    const auto date = FormatLocalNow("%m/%d/%y");
    const auto time = FormatLocalNow("%H:%M:%S");

    std::vector<std::uint8_t> description;
    description.reserve(2U + worldName.size() + 2U + date.size() + 1U +
                        time.size() + 4U + 3U);
    AppendAscii(description, "WS");
    description.insert(description.end(), worldName.begin(), worldName.end());
    description.insert(description.end(), {0xD4U, 0xDAU}); // CP936 在
    description.insert(description.end(), date.begin(), date.end());
    description.push_back(' ');
    description.insert(description.end(), time.begin(), time.end());
    description.insert(description.end(), {0xC1U, 0xACU, 0xBDU, 0xD3U}); // CP936 连接
    AppendAscii(description, "LS.");
    return description;
}

std::vector<std::uint8_t> GameServerConnectedDescription(std::int32_t worldId)
{
    const auto date = FormatLocalNow("%m/%d/%y");
    const auto time = FormatLocalNow("%H:%M:%S");
    const std::string worldNumber = std::to_string(worldId);

    std::vector<std::uint8_t> description;
    description.reserve(2U + 2U + date.size() + 1U + time.size() + 4U + 3U +
                        worldNumber.size() + 2U);
    AppendAscii(description, "GS");
    description.insert(description.end(), {0xD4U, 0xDAU}); // CP936 在
    description.insert(description.end(), date.begin(), date.end());
    description.push_back(' ');
    description.insert(description.end(), time.begin(), time.end());
    description.insert(description.end(), {0xC1U, 0xACU, 0xBDU, 0xD3U}); // CP936 连接
    AppendAscii(description, "WS(");
    AppendAscii(description, worldNumber);
    AppendAscii(description, ").");
    return description;
}
}

ServerMessageHandler::ServerMessageHandler(IServerMessageContext& context) noexcept
    : m_Context(context)
{
}

void ServerMessageHandler::OnServerMessage(LoginNet::CMessage& message)
{
    switch (message.MessageType()) {
    case kWorldConnectedMessageType:
        OnWorldConnected(message);
        break;
    case kWorldDisconnectedMessageType:
        OnWorldDisconnected(message);
        break;
    case kWorldUsersSnapshotMessageType:
        OnWorldUsersSnapshot(message);
        break;
    case kClearAccountsMessageType:
        OnAccountsCleared(message);
        break;
    case kWorldTelemetryMessageType:
        OnWorldTelemetry(message);
        break;
    case kGameServerConnectedLogMessageType:
        OnGameServerConnectedLog(message);
        break;
    case kWorldServerLogMessageType:
        OnWorldServerLog(message);
        break;
    case kTypedServerLogMessageType:
        OnTypedServerLog(message);
        break;
    default:
        break;
    }
}

void ServerMessageHandler::OnWorldConnected(LoginNet::CMessage& message)
{
    const std::int32_t worldId = message.Base().GetLong();
    const auto worldName =
        message.Base().GetStrBytes(kServerStringLimit).value_or(
            std::vector<std::uint8_t>{});

    m_Context.SetWorldSocketMapID(message.SocketID(), worldId);
    static_cast<void>(m_Context.AddWorld(worldId, worldName));
    m_Context.QueueWorldConnectedOperatorLog(worldName);

    LoginNet::CMessage acknowledgement(kWorldConnectedAckMessageType);
    acknowledgement.Base().Add(m_Context.AreaID());
    m_Context.SendToWorldSocket(acknowledgement, message.SocketID());

    if (ServerInfoLogEnabled()) {
        m_Context.PushServerInfoLog(ServLog{
            .sourceIp = message.IP(),
            .serverType = -2,
            .serverNumber = worldId,
            .description = WorldConnectedDescription(worldName),
        });
    }
}

void ServerMessageHandler::OnWorldDisconnected(const LoginNet::CMessage& message)
{
    const std::int32_t worldId = message.MapID();
    if (const auto worldName = m_Context.WorldNameByID(worldId)) {
        m_Context.QueueWorldLostOperatorLog(*worldName);
        m_Context.ClearCdkeysByWorldID(worldId);
    }
    static_cast<void>(m_Context.DelWorld(worldId));
}

void ServerMessageHandler::OnWorldUsersSnapshot(LoginNet::CMessage& message)
{
    const std::int32_t worldId = message.Base().GetLong();
    if (worldId == 0) {
        return;
    }

    const std::int32_t declaredCount = message.Base().GetLong();
    if (declaredCount == 0) {
        return;
    }

    std::vector<std::vector<std::uint8_t>> accounts;
    std::int32_t remaining = declaredCount;
    while (remaining > 0) {
        auto account =
            message.Base().GetStrBytes(kServerStringLimit).value_or(
                std::vector<std::uint8_t>{});
        static_cast<void>(m_Context.AddCdkey(account, worldId));
        accounts.push_back(std::move(account));
        --remaining;
    }

    m_Context.QueueOnlineUserDatabaseUpdate(worldId, std::move(accounts));
}

void ServerMessageHandler::OnAccountsCleared(LoginNet::CMessage& message)
{
    std::int32_t remaining = message.Base().GetLong();
    while (remaining > 0) {
        const auto account =
            message.Base().GetStrBytes(kServerStringLimit).value_or(
                std::vector<std::uint8_t>{});
        m_Context.ClearCdkey(account);
        --remaining;
    }
}

void ServerMessageHandler::OnWorldTelemetry(LoginNet::CMessage& message)
{
    PingWorldServerInfo worldInfo{
        .ip = LegacyIPv4Text(message.IP()),
        .port = std::bit_cast<std::uint32_t>(message.MapID()),
        .playerCount = std::bit_cast<std::uint32_t>(message.Base().GetLong()),
        .gameServers = {},
    };

    std::int32_t remaining = message.Base().GetLong();
    while (remaining > 0) {
        PingGameServerInfo gameInfo;
        gameInfo.ip =
            message.Base().GetStrBytes(kGameServerIpLimit).value_or(
                std::vector<std::uint8_t>{});
        gameInfo.port = std::bit_cast<std::uint32_t>(message.Base().GetLong());
        gameInfo.playerCount =
            std::bit_cast<std::uint32_t>(message.Base().GetLong());
        worldInfo.gameServers.push_back(std::move(gameInfo));
        --remaining;
    }

    m_Context.AppendPingWorldServerInfo(std::move(worldInfo));
}

void ServerMessageHandler::OnGameServerConnectedLog(LoginNet::CMessage& message)
{
    if (!ServerInfoLogEnabled()) {
        return;
    }

    const std::uint32_t sourceIp =
        std::bit_cast<std::uint32_t>(message.Base().GetLong());
    const std::int32_t serverNumber = message.Base().GetLong();
    const std::int32_t serverType = message.MapID();
    m_Context.PushServerInfoLog(ServLog{
        .sourceIp = sourceIp,
        .serverType = serverType,
        .serverNumber = serverNumber,
        .description = GameServerConnectedDescription(serverType),
    });
}

void ServerMessageHandler::OnWorldServerLog(LoginNet::CMessage& message)
{
    if (!ServerInfoLogEnabled()) {
        return;
    }
    OnSuppliedServerLog(message, -2);
}

void ServerMessageHandler::OnTypedServerLog(LoginNet::CMessage& message)
{
    if (!ServerInfoLogEnabled()) {
        return;
    }

    const std::int8_t serverType =
        std::bit_cast<std::int8_t>(message.Base().GetByte());
    OnSuppliedServerLog(message, static_cast<std::int32_t>(serverType));
}

void ServerMessageHandler::OnSuppliedServerLog(LoginNet::CMessage& message,
                                                std::int32_t serverType)
{
    const std::uint32_t sourceIp =
        std::bit_cast<std::uint32_t>(message.Base().GetLong());
    const std::int32_t serverNumber = message.Base().GetLong();
    auto description =
        message.Base().GetStrBytes(kServerLogTextLimit).value_or(
            std::vector<std::uint8_t>{});

    m_Context.PushServerInfoLog(ServLog{
        .sourceIp = sourceIp,
        .serverType = serverType,
        .serverNumber = serverNumber,
        .description = std::move(description),
    });
}

bool ServerMessageHandler::ServerInfoLogEnabled() const noexcept
{
    const auto configured = m_Context.ServerInfoLogTime();
    return configured.has_value() && *configured != 0U;
}
}
