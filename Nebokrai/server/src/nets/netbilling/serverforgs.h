#pragma once

#include "../msgqueue.h"
#include "../servers.h"
#include "clientforgs.h"
#include "message.h"

#include <asio.hpp>

#include <cstdint>
#include <memory>

/*
 * Исходный владелец: nets/netbilling/serverforgs.cpp / .h
 *
 * Это тонкий Billing-владелец общего Asio CServer: component defaults,
 * фабрика CClientForGS и FIFO сообщений. Доменная обработка выполняется
 * Billing::CGame, а не сетевым потоком.
 */
namespace BillingNet
{
class CServerForGS final : public CServer
{
public:
    explicit CServerForGS(asio::any_io_executor executor,
                          std::uint32_t nowMs = 0);

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
