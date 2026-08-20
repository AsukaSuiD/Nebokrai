#pragma once

#include "../msgqueue.h"
#include "../servers.h"
#include "message.h"
#include "mynetserverclient_world.h"

#include <asio.hpp>

#include <cstdint>
#include <memory>
#include <optional>

/*
 * Исходный владелец: nets/netlogin/mynetserver_world.cpp
 *
 * LoginServer EXE/PDB: конструктор 0x0006A9B0, CreateServerClient 0x0006A9F0;
 * связанные CGame::InitNetServer_World 0x2F90 и ReLoadSetup 0xF4E0.
 *
 * По умолчанию: не более 100 незавершённых отправок, ограничение отправки
 * на клиента = 0x1000000.
 * World parser всегда проверяет content CRC независимо от bWorldCheckMsgCon;
 * parser errors не ставят IP-ban/QUIT. Общий receive-rate guard при включении
 * сохраняет собственный ban/QUIT.
 */
namespace LoginNet
{
class CMyNetServerWorld final : public CServer
{
public:
    explicit CMyNetServerWorld(asio::any_io_executor executor,
                               std::uint32_t nowMs = 0);

    void ConfigureReceive(bool enabled,
                          std::uint32_t maximumBytesPerSecond,
                          std::uint32_t forbidTimeMs) noexcept;
    void ConfigureConnectionLimits(std::int32_t maximumClients,
                                   std::int32_t maximumInFlightSends,
                                   std::int32_t permittedSendBytes) noexcept;

    [[nodiscard]] std::int32_t PendingMessages() const;
    [[nodiscard]] std::unique_ptr<CMessage> PopReceivedMessage();

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
};
}
