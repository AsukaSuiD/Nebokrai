#pragma once

#include "../msgqueue.h"
#include "../servers.h"
#include "message.h"
#include "mynetserverclient_client.h"

#include <asio.hpp>

#include <cstdint>
#include <deque>
#include <memory>
#include <optional>
#include <span>
#include <variant>
#include <vector>

/*
 * Owner: nets/netlogin/mynetserver_client.cpp
 *
 * LoginServer EXE/PDB: ctor 0x0006A870, CreateServerClient 0x0006A8B0,
 * OnMapStrError 0x0006A910; связанные CGame::InitNetServer_Client 0x2DA0 и
 * ReLoadSetup 0xF4E0.
 *
 * Component defaults: max in-flight sends = 5, per-client send limit =
 * 0x400000. Setup отдельно задаёт receive-rate/ban, две CRC-проверки, max
 * frame, max clients, send limits, поздний backlog и first-receive timeout.
 *
 * Ошибки parser-а с доказанной политикой возвращают ForbidAndQuit общему
 * CServer. Остальные malformed-границы остаются snapshot errors без новой
 * политики. OnMapStrError публикует synthetic 0x10001 с CD-key.
 */
namespace LoginNet
{
struct ClientOneMessageSizeExceeded
{
    std::int32_t socketId{};
    std::vector<std::uint8_t> cdkey;
    std::uint32_t peerIPv4{};
    std::int32_t declared{};
    std::int32_t permitted{};
};

struct ClientTotalMessageSizeExceeded
{
    std::int32_t socketId{};
    std::vector<std::uint8_t> cdkey;
    std::uint32_t peerIPv4{};
    std::int32_t actual{};
    std::int32_t permitted{};
};

using ClientNetworkNotice = std::variant<ClientOneMessageSizeExceeded,
                                         ClientTotalMessageSizeExceeded>;

class CMyNetServerClient final : public CServer
{
public:
    explicit CMyNetServerClient(asio::any_io_executor executor,
                                std::uint32_t nowMs = 0);

    void ConfigureReceive(bool checkLengthCrc,
                          bool checkContentCrc,
                          std::uint32_t maximumBytesPerSecond,
                          std::uint32_t forbidTimeMs,
                          std::uint32_t maximumMessageLength) noexcept;
    void ConfigureConnectionLimits(std::int32_t maximumClients,
                                   std::int32_t maximumInFlightSends,
                                   std::int32_t permittedSendBytes) noexcept;

    [[nodiscard]] std::int32_t PendingMessages() const;
    [[nodiscard]] std::unique_ptr<CMessage> PopReceivedMessage();
    [[nodiscard]] std::optional<ClientNetworkNotice> PopNotice();

protected:
    [[nodiscard]] std::unique_ptr<CServerClient>
    CreateServerClient(std::int32_t socketId,
                       std::uint32_t peerIPv4,
                       std::uint32_t nowMs) override;
    void OnAccepted(CServerClient& client) override;
    [[nodiscard]] ReceiveCallbackResult OnReceive(CServerClient& client,
                                                  std::uint32_t nowMs) override;
    void OnClose(CServerClient& client) override;
    void OnReceiveRateExceeded(CServerClient& client,
                               std::int32_t actual,
                               std::int32_t permitted) override;
    void OnMissingMapIDClient(std::int32_t mapId, std::int32_t socketId) override;
    void OnMissingMapNameClient(std::span<const std::uint8_t> mapName,
                                std::int32_t socketId) override;

private:
    CMsgQueue<CMessage> m_ReceivedMessages;
    ClientReceiveSettings m_ReceiveSettings{};
    std::deque<ClientNetworkNotice> m_Notices;
};
}
