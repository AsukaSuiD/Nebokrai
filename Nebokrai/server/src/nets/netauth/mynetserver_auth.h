#pragma once

#include "../msgqueue.h"
#include "../servers.h"
#include "message.h"
#include "mynetserverclient_auth.h"

#include <asio.hpp>

#include <cstdint>
#include <memory>

/*
 * Owner: nets/netauth/mynetserver_auth.cpp
 *
 * AuthServer EXE/PDB: ctor 0x00012770, CreateServerClient 0x000127B0;
 * связанные CGame::InitNetServer_Auth 0x00002AF0 и ProcessMessage 0x00002090.
 * Производный default меняет max in-flight sends на 100 и per-client send
 * buffer на 0x1000000. Поздний CGame затем применяет setup-пределы, максимум
 * LoginServer, поздний backlog 10 и timeout первого пакета.
 *
 * Этот owner связывает общий CServer с Auth CMessage FIFO и конкретным
 * CMyNetServerClientAuth. Transport остаётся в общем Asio CServer; доменные
 * обработчики Auth сообщений здесь не исполняются.
 */
class CMyNetServerAuth final : public CServer
{
public:
    explicit CMyNetServerAuth(asio::any_io_executor executor, std::uint32_t nowMs = 0);

    void ConfigureLimits(std::int32_t maxLoginServers,
                         std::int32_t maxInFlightSends,
                         std::int32_t permittedSendBytes,
                         std::int32_t newAcceptTimeoutMs) noexcept;

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
