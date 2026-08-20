#pragma once

#include "../clients.h"
#include "../msgqueue.h"
#include "message.h"

#include <asio.hpp>

#include <array>
#include <cstdint>
#include <memory>
#include <optional>
#include <span>
#include <vector>

/*
 * Исходный владелец: nets/networld/mynetclient.cpp / .h.
 *
 * Исходящее соединение WorldServer с LoginServer. EXE/PDB подтверждают тот же
 * 12-байтовый CRC-envelope, собственную FIFO, partial TCP accumulator и
 * synthetic close 0x3FC01. Asio заменяет только WinSock transport.
 */
namespace WorldNet
{
enum class ClientReceiveErrorKind
{
    LengthChecksumMismatch,
    SignedFrameLengthReactionUnknown,
    ShortFrameReactionUnknown,
    MessageCreateFailed,
    ContentChecksumMismatch,
    PendingSizeOverflowReactionUnknown,
};

struct ClientReceiveError
{
    ClientReceiveErrorKind kind{};
    std::uint32_t declared{};
    std::uint32_t expected{};
    std::uint32_t actual{};
    std::int32_t messageType{};
    std::optional<CreateMessageError> messageError;
};

struct ClientReadResult
{
    std::size_t messages{};
    bool closed{};
    std::optional<ClientReceiveError> receiveError;
    std::error_code ioError;
    [[nodiscard]] explicit operator bool() const noexcept
    {
        return !receiveError.has_value() && !ioError;
    }
};

class CMyNetClient
{
public:
    explicit CMyNetClient(asio::any_io_executor executor);
    ~CMyNetClient();

    CMyNetClient(const CMyNetClient&) = delete;
    CMyNetClient& operator=(const CMyNetClient&) = delete;

    [[nodiscard]] asio::awaitable<ClientConnectResult>
    Connect(const asio::ip::tcp::endpoint& remote,
            std::optional<asio::ip::tcp::endpoint> local = std::nullopt);
    [[nodiscard]] std::int32_t Close();
    void HandleTransportClose();

    [[nodiscard]] bool IsConnected() const noexcept;
    [[nodiscard]] ClientSendQueue& SendQueue() noexcept;
    [[nodiscard]] asio::awaitable<ClientFlushResult> FlushOutgoing();
    [[nodiscard]] asio::awaitable<ClientReadResult> ReadOnce();
    [[nodiscard]] std::optional<ClientReadResult> PollReadOnce();
    [[nodiscard]] std::optional<ClientReceiveError>
    AcceptReceivedBytes(std::span<const std::uint8_t> received,
                        std::size_t& acceptedMessages,
                        std::uint32_t receiveTimeMs);

    void PushMessage(std::unique_ptr<CMessage> message);
    [[nodiscard]] std::vector<std::unique_ptr<CMessage>> TakeAllMessages();
    [[nodiscard]] std::int32_t PendingMessages() const;

private:
    void DiscardPending() noexcept;

    asio::ip::tcp::socket m_Socket;
    bool m_Connected{};
    ClientSendQueue m_SendQueue;
    std::vector<std::uint8_t> m_ReceiveBuffer;
    CMsgQueue<CMessage> m_Messages;
};
}
