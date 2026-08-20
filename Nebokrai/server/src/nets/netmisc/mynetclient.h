#pragma once

#include "../clients.h"
#include "../msgqueue.h"
#include "message.h"

#include <asio.hpp>

#include <array>
#include <cstdint>
#include <memory>
#include <optional>
#include <vector>

/*
 * Исходный владелец: nets/netmisc/mynetclient.cpp / .h.
 *
 * MiscServer EXE/PDB: входной поток состоит из CRC-envelope, все полные кадры
 * одного чтения попадают в одну FIFO, а transport close публикует 0x16EA01.
 * Общая Asio-очередь заменяет только WinSock-потоки и WSAEventSelect.
 */
namespace MiscNet
{
enum class MiscClientReceiveErrorKind
{
    LengthChecksumMismatch,
    SignedFrameLengthReactionUnknown,
    ShortFrameReactionUnknown,
    MessageCreateFailed,
    ContentChecksumMismatch,
    PendingSizeOverflowReactionUnknown,
};

struct MiscClientReceiveError
{
    MiscClientReceiveErrorKind kind{};
    std::uint32_t declared{};
    std::uint32_t expected{};
    std::uint32_t actual{};
    std::int32_t messageType{};
    std::optional<CreateMessageError> messageError;
};

struct MiscClientReadResult
{
    std::size_t messages{};
    bool closed{};
    std::optional<MiscClientReceiveError> receiveError;
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
    [[nodiscard]] std::optional<asio::ip::tcp::endpoint> ConnectedEndpoint() const noexcept;
    void EnableControlSend() noexcept;
    [[nodiscard]] ClientSendQueue& SendQueue() noexcept;
    [[nodiscard]] const ClientSendQueue& SendQueue() const noexcept;

    [[nodiscard]] asio::awaitable<ClientFlushResult> FlushOutgoing();
    [[nodiscard]] asio::awaitable<MiscClientReadResult> ReadOnce();
    [[nodiscard]] std::optional<MiscClientReadResult> PollReadOnce();
    [[nodiscard]] std::optional<MiscClientReceiveError>
    AcceptReceivedBytes(std::span<const std::uint8_t> received,
                        std::size_t& acceptedMessages,
                        std::uint32_t receiveTimeMs);

    void PushMessage(std::unique_ptr<CMessage> message);
    [[nodiscard]] std::vector<std::unique_ptr<CMessage>> TakeAllMessages();
    [[nodiscard]] std::int32_t PendingMessages() const;
    [[nodiscard]] std::size_t PendingBytes() const noexcept;

private:
    void DiscardPending() noexcept;

    asio::ip::tcp::socket m_Socket;
    bool m_Connected{};
    bool m_ControlSend{};
    std::optional<asio::ip::tcp::endpoint> m_ConnectedEndpoint;
    ClientSendQueue m_SendQueue;
    std::vector<std::uint8_t> m_ReceiveBuffer;
    CMsgQueue<CMessage> m_Messages;
};
}
