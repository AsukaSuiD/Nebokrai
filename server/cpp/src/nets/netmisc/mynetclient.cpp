#include "mynetclient.h"

#include "../../public/crc32static.h"

#include <asio/redirect_error.hpp>
#include <asio/use_awaitable.hpp>

#include <bit>
#include <chrono>
#include <limits>

namespace MiscNet
{
namespace
{
constexpr std::size_t kServerEnvelopeSize = 12;
constexpr std::size_t kInnerMessageHeaderSize = 16;
constexpr std::size_t kMinimumServerFrame = kServerEnvelopeSize + kInnerMessageHeaderSize;
constexpr std::int32_t kCloseMessageType = 0x0016'EA01;

std::uint32_t ReadU32(std::span<const std::uint8_t> bytes) noexcept
{
    return static_cast<std::uint32_t>(bytes[0]) |
           (static_cast<std::uint32_t>(bytes[1]) << 8U) |
           (static_cast<std::uint32_t>(bytes[2]) << 16U) |
           (static_cast<std::uint32_t>(bytes[3]) << 24U);
}

std::uint32_t MonotonicMilliseconds() noexcept
{
    return static_cast<std::uint32_t>(
        std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::steady_clock::now().time_since_epoch()).count());
}
}

CMyNetClient::CMyNetClient(asio::any_io_executor executor)
    : m_Socket(std::move(executor))
{
    m_ReceiveBuffer.reserve(kClientInitialReceiveCapacity);
}

CMyNetClient::~CMyNetClient()
{
    asio::error_code ignored;
    if (m_Socket.is_open()) {
        m_Socket.shutdown(asio::ip::tcp::socket::shutdown_both, ignored);
        m_Socket.close(ignored);
    }
}

asio::awaitable<ClientConnectResult>
CMyNetClient::Connect(const asio::ip::tcp::endpoint& remote,
                      std::optional<asio::ip::tcp::endpoint> local)
{
    if (m_Socket.is_open()) {
        asio::error_code ignored;
        m_Socket.close(ignored);
    }

    asio::error_code error;
    m_Socket.open(asio::ip::tcp::v4(), error);
    if (error) {
        co_return ClientConnectResult{ClientConnectStatus::IoError, error};
    }
    if (local) {
        m_Socket.bind(*local, error);
        if (error) {
            asio::error_code ignored;
            m_Socket.close(ignored);
            co_return ClientConnectResult{ClientConnectStatus::IoError, error};
        }
    }

    ClientConnectResult result = co_await ConnectTcpIPv4(m_Socket, remote);
    if (result.status == ClientConnectStatus::Connected) {
        m_Connected = true;
        m_ConnectedEndpoint = remote;
    } else {
        asio::error_code ignored;
        m_Socket.close(ignored);
    }
    co_return result;
}

std::int32_t CMyNetClient::Close()
{
    asio::error_code ignored;
    if (m_Socket.is_open()) {
        m_Socket.shutdown(asio::ip::tcp::socket::shutdown_both, ignored);
        m_Socket.close(ignored);
    }
    m_Connected = false;
    m_ConnectedEndpoint.reset();
    return 1;
}

void CMyNetClient::HandleTransportClose()
{
    static_cast<void>(Close());
    PushMessage(std::make_unique<CMessage>(kCloseMessageType));
}

bool CMyNetClient::IsConnected() const noexcept { return m_Connected; }

std::optional<asio::ip::tcp::endpoint> CMyNetClient::ConnectedEndpoint() const noexcept
{
    return m_ConnectedEndpoint;
}

void CMyNetClient::EnableControlSend() noexcept { m_ControlSend = true; }
ClientSendQueue& CMyNetClient::SendQueue() noexcept { return m_SendQueue; }
const ClientSendQueue& CMyNetClient::SendQueue() const noexcept { return m_SendQueue; }

asio::awaitable<ClientFlushResult> CMyNetClient::FlushOutgoing()
{
    if (!m_Connected || !m_ControlSend) {
        co_return ClientFlushResult{ClientFlushStatus::IoError,
                                    0,
                                    0,
                                    std::make_error_code(std::errc::not_connected)};
    }
    co_return co_await m_SendQueue.Flush(m_Socket);
}

asio::awaitable<MiscClientReadResult> CMyNetClient::ReadOnce()
{
    if (!m_Connected) {
        co_return MiscClientReadResult{
            .ioError = std::make_error_code(std::errc::not_connected)};
    }

    std::array<std::uint8_t, kClientMaxReceiveChunk> buffer{};
    asio::error_code error;
    const std::size_t received = co_await m_Socket.async_read_some(
        asio::buffer(buffer), asio::redirect_error(asio::use_awaitable, error));
    if (error) {
        co_return MiscClientReadResult{.ioError = error};
    }
    if (received == 0U) {
        HandleTransportClose();
        co_return MiscClientReadResult{.closed = true};
    }

    std::size_t accepted = 0;
    auto receiveError = AcceptReceivedBytes(
        std::span<const std::uint8_t>(buffer.data(), received),
        accepted,
        MonotonicMilliseconds());
    co_return MiscClientReadResult{.messages = accepted,
                                   .receiveError = std::move(receiveError)};
}

std::optional<MiscClientReadResult> CMyNetClient::PollReadOnce()
{
    if (!m_Connected) return std::nullopt;
    asio::error_code modeError;
    m_Socket.non_blocking(true, modeError);
    if (modeError) return MiscClientReadResult{.ioError = modeError};

    std::array<std::uint8_t, kClientMaxReceiveChunk> buffer{};
    asio::error_code error;
    const std::size_t received = m_Socket.read_some(asio::buffer(buffer), error);
    if (error == asio::error::would_block || error == asio::error::try_again) {
        return std::nullopt;
    }
    if (error == asio::error::eof || received == 0U) {
        HandleTransportClose();
        return MiscClientReadResult{.closed = true};
    }
    if (error) return MiscClientReadResult{.ioError = error};

    std::size_t accepted = 0;
    auto receiveError = AcceptReceivedBytes(
        std::span<const std::uint8_t>(buffer.data(), received),
        accepted,
        MonotonicMilliseconds());
    return MiscClientReadResult{.messages = accepted,
                                .receiveError = std::move(receiveError)};
}

std::optional<MiscClientReceiveError>
CMyNetClient::AcceptReceivedBytes(std::span<const std::uint8_t> received,
                                  std::size_t& acceptedMessages,
                                  std::uint32_t receiveTimeMs)
{
    acceptedMessages = 0;
    if (received.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) ||
        m_ReceiveBuffer.size() >
            static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) - received.size()) {
        return MiscClientReceiveError{
            .kind = MiscClientReceiveErrorKind::PendingSizeOverflowReactionUnknown};
    }

    m_ReceiveBuffer.insert(m_ReceiveBuffer.end(), received.begin(), received.end());
    std::size_t consumed = 0;
    while (m_ReceiveBuffer.size() - consumed >= kServerEnvelopeSize) {
        const std::span<const std::uint8_t> frame(
            m_ReceiveBuffer.data() + consumed, m_ReceiveBuffer.size() - consumed);
        const std::uint32_t declared = ReadU32(frame.first<4>());
        const std::uint32_t expectedLength = ReadU32(frame.subspan<4, 4>());
        const std::uint32_t actualLength = DataCrc32(frame.first<4>());
        if (actualLength != expectedLength) {
            DiscardPending();
            return MiscClientReceiveError{
                .kind = MiscClientReceiveErrorKind::LengthChecksumMismatch,
                .declared = declared,
                .expected = expectedLength,
                .actual = actualLength};
        }
        if (std::bit_cast<std::int32_t>(declared) < 0) {
            DiscardPending();
            return MiscClientReceiveError{
                .kind = MiscClientReceiveErrorKind::SignedFrameLengthReactionUnknown,
                .declared = declared};
        }

        const std::size_t frameLength = static_cast<std::size_t>(declared);
        if (frame.size() < frameLength) {
            break;
        }
        if (frameLength == kServerEnvelopeSize) {
            DiscardPending();
            return MiscClientReceiveError{
                .kind = MiscClientReceiveErrorKind::MessageCreateFailed,
                .declared = declared,
                .messageError = CreateMessageError{CreateMessageErrorKind::EmptyInput,
                                                   std::nullopt}};
        }
        if (frameLength < kMinimumServerFrame) {
            DiscardPending();
            return MiscClientReceiveError{
                .kind = MiscClientReceiveErrorKind::ShortFrameReactionUnknown,
                .declared = declared};
        }

        CreateMessageResult created = CMessage::CreateMessageWithoutRLE(
            frame.subspan(kServerEnvelopeSize, frameLength - kServerEnvelopeSize),
            receiveTimeMs);
        if (!created) {
            DiscardPending();
            return MiscClientReceiveError{
                .kind = MiscClientReceiveErrorKind::MessageCreateFailed,
                .declared = declared,
                .messageError = created.error};
        }
        const std::uint32_t expectedContent = ReadU32(frame.subspan<8, 4>());
        const std::uint32_t actualContent = DataCrc32(created.message->WireBytes());
        if (actualContent != expectedContent) {
            const std::int32_t type = created.message->MessageType();
            DiscardPending();
            return MiscClientReceiveError{
                .kind = MiscClientReceiveErrorKind::ContentChecksumMismatch,
                .declared = declared,
                .expected = expectedContent,
                .actual = actualContent,
                .messageType = type};
        }

        PushMessage(std::move(created.message));
        consumed += frameLength;
        ++acceptedMessages;
    }

    if (consumed != 0U) {
        m_ReceiveBuffer.erase(m_ReceiveBuffer.begin(),
                              m_ReceiveBuffer.begin() +
                                  static_cast<std::ptrdiff_t>(consumed));
    }
    return std::nullopt;
}

void CMyNetClient::PushMessage(std::unique_ptr<CMessage> message)
{
    if (message) {
        static_cast<void>(m_Messages.PushMessage(std::move(message)));
    }
}

std::vector<std::unique_ptr<CMessage>> CMyNetClient::TakeAllMessages()
{
    std::vector<std::unique_ptr<CMessage>> messages;
    auto queue = m_Messages.GetAllMessage();
    messages.reserve(queue.size());
    while (!queue.empty()) {
        messages.push_back(std::move(queue.front()));
        queue.pop_front();
    }
    return messages;
}

std::int32_t CMyNetClient::PendingMessages() const { return m_Messages.GetSize(); }
std::size_t CMyNetClient::PendingBytes() const noexcept { return m_ReceiveBuffer.size(); }
void CMyNetClient::DiscardPending() noexcept { m_ReceiveBuffer.clear(); }
}
