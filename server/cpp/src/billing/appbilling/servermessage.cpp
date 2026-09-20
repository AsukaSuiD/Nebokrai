#include "servermessage.h"

#include "../../nets/netbilling/message.h"
#include "../../nets/netbilling/serverforgs.h"

#include <spdlog/spdlog.h>

#include <array>
#include <bit>
#include <string>
#include <vector>

namespace Billing
{
namespace
{
constexpr std::int32_t kGameServerConnected = 0x000E'F101;
constexpr std::int32_t kGameServerDisconnected = 0x0010'EF01;
constexpr std::size_t kAddressLimit = 0x20;

std::string BytesToString(std::span<const std::uint8_t> bytes)
{
    return {bytes.begin(), bytes.end()};
}

std::string FormatIPv4(std::uint32_t address)
{
    const auto bytes = std::bit_cast<std::array<std::uint8_t, 4>>(address);
    return std::to_string(bytes[0]) + "." + std::to_string(bytes[1]) + "." +
           std::to_string(bytes[2]) + "." + std::to_string(bytes[3]);
}
}

ServerMessageOutcome OnServerMessage(BillingNet::CMessage& message,
                                     BillingNet::CServerForGS* server)
{
    if (message.MessageType() == kGameServerConnected) {
        if (server != nullptr) {
            static_cast<void>(server->SetClientMapID(message.SocketID(), message.SocketID()));
        }
        spdlog::info("BillingServer: GameServer {} подключён с адреса {}",
                     message.SocketID(), FormatIPv4(message.IP()));
        return {.kind = ServerMessageOutcome::Kind::Connected};
    }

    if (message.MessageType() == kGameServerDisconnected) {
        const std::int32_t mapId = message.Base().GetLong();
        auto addressBytes = message.Base().GetStrBytes(kAddressLimit)
                                .value_or(std::vector<std::uint8_t>{});
        const std::int32_t port = message.Base().GetLong();
        const bool allowed = server != nullptr &&
            server->IsAllowedAddress(addressBytes, static_cast<std::uint16_t>(port));
        spdlog::info("BillingServer: {}GameServer {}<{}:{}> отключён",
                     allowed ? "" : "неразрешённый ",
                     mapId,
                     BytesToString(addressBytes),
                     port);
        return {.kind = ServerMessageOutcome::Kind::Disconnected, .allowed = allowed};
    }
    return {};
}
}
