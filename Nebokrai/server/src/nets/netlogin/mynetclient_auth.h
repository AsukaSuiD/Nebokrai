#pragma once

#include "../clients.h"
#include "../msgqueue.h"
#include "message.h"

#include <asio.hpp>

#include <array>
#include <cstdint>
#include <memory>
#include <optional>
#include <variant>
#include <vector>

/*
 * Исходный владелец: nets/netlogin/mynetclient_auth.cpp / .h
 *
 * LoginServer EXE/PDB: конструктор 0x0006CAC0, HandleClose 0x0006CAF0,
 * OnReceive 0x0006CB90.
 *
 * Исходящий Auth путь использует общий CClient send contract и собственный
 * receive accumulator 0x100000. Успешный connect только поднимает connect flag;
 * явный Close не создаёт synthetic message. Transport close через OnClose /
 * HandleClose публикует 0xCF301 в той же FIFO, что обычные Auth сообщения.
 *
 * Формат приёма: [len, crc(len), crc(нормализованное сообщение), сообщение].
 * Длина CRC
 * проверяется сразу после 12 bytes; malformed sign/short length сохраняются
 * отдельными неизвестностями. Reconnect synthetic 0xCF302 + raw pointer в
 * оригинале заменён типизированным событием в той же FIFO.
 */
namespace LoginNet
{
class CMyNetClientAuth;

struct AuthClientReconnected
{
    std::shared_ptr<CMyNetClientAuth> client;
};

using AuthClientEventPayload = std::variant<std::unique_ptr<CMessage>,
                                            AuthClientReconnected>;

struct AuthClientEvent
{
    AuthClientEventPayload payload;
};

class AuthClientEventPublisher
{
public:
    AuthClientEventPublisher() = default;

    void PublishMessage(std::unique_ptr<CMessage> message) const;
    void PublishReconnected(std::shared_ptr<CMyNetClientAuth> client) const;
    [[nodiscard]] explicit operator bool() const noexcept;

private:
    friend class CMyNetClientAuth;
    explicit AuthClientEventPublisher(std::shared_ptr<CMsgQueue<AuthClientEvent>> events);

    std::shared_ptr<CMsgQueue<AuthClientEvent>> m_Events;
};

enum class AuthClientReceiveErrorKind
{
    LengthChecksumMismatch,
    SignedFrameLengthReactionUnknown,
    ShortFrameReactionUnknown,
    MessageCreateFailed,
    ContentChecksumMismatch,
    PendingSizeOverflowReactionUnknown,
};

struct AuthClientReceiveError
{
    AuthClientReceiveErrorKind kind{};
    std::uint32_t declared{};
    std::uint32_t expected{};
    std::uint32_t actual{};
    std::int32_t messageType{};
    std::optional<CreateMessageError> messageError;
};

struct AuthClientReadResult
{
    std::size_t messages{};
    bool closed{};
    std::optional<AuthClientReceiveError> receiveError;
    std::error_code ioError;

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return !receiveError.has_value() && !ioError;
    }
};

class CMyNetClientAuth : public std::enable_shared_from_this<CMyNetClientAuth>
{
public:
    explicit CMyNetClientAuth(asio::any_io_executor executor);
    ~CMyNetClientAuth();

    CMyNetClientAuth(const CMyNetClientAuth&) = delete;
    CMyNetClientAuth& operator=(const CMyNetClientAuth&) = delete;

    [[nodiscard]] asio::awaitable<ClientConnectResult>
    Connect(const asio::ip::tcp::endpoint& remote);
    [[nodiscard]] std::int32_t Close();
    void HandleTransportClose();

    [[nodiscard]] bool IsConnected() const noexcept;
    [[nodiscard]] std::optional<asio::ip::tcp::endpoint> ConnectedEndpoint() const noexcept;

    void EnableControlSend() noexcept;
    [[nodiscard]] ClientSendQueue& SendQueue() noexcept;
    [[nodiscard]] const ClientSendQueue& SendQueue() const noexcept;

    [[nodiscard]] asio::awaitable<ClientFlushResult> FlushOutgoing();
    [[nodiscard]] asio::awaitable<AuthClientReadResult> ReadOnce();
    [[nodiscard]] std::optional<AuthClientReceiveError>
    AcceptReceivedBytes(std::span<const std::uint8_t> received,
                        std::size_t& acceptedMessages);

    [[nodiscard]] std::int32_t PendingEvents() const;
    [[nodiscard]] std::unique_ptr<AuthClientEvent> PopEvent();
    void PublishReconnected(std::shared_ptr<CMyNetClientAuth> client);
    [[nodiscard]] AuthClientEventPublisher EventPublisher() const;
    [[nodiscard]] std::size_t PendingBytes() const noexcept;

private:
    void PublishMessage(std::unique_ptr<CMessage> message);
    void DiscardPending() noexcept;

    asio::ip::tcp::socket m_Socket;
    bool m_Connected{};
    bool m_ControlSend{};
    std::optional<asio::ip::tcp::endpoint> m_ConnectedEndpoint;
    ClientSendQueue m_SendQueue;
    std::vector<std::uint8_t> m_ReceiveBuffer;
    std::shared_ptr<CMsgQueue<AuthClientEvent>> m_Events;
};
}
