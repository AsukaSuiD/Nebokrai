#include "mynetclient_auth.h"

#include "../../public/crc32static.h"

#include <asio/redirect_error.hpp>
#include <asio/use_awaitable.hpp>

#include <bit>
#include <limits>
#include <utility>

namespace LoginNet
{
namespace
{
constexpr std::size_t kServerEnvelopeSize = 12;
constexpr std::size_t kInnerMessageHeaderSize = 16;
constexpr std::size_t kMinimumServerFrame = kServerEnvelopeSize + kInnerMessageHeaderSize;
constexpr std::int32_t kCloseMessageType = 0x000CF301;

std::uint32_t ReadU32(std::span<const std::uint8_t> bytes) noexcept
{
    return static_cast<std::uint32_t>(bytes[0]) |
           (static_cast<std::uint32_t>(bytes[1]) << 8U) |
           (static_cast<std::uint32_t>(bytes[2]) << 16U) |
           (static_cast<std::uint32_t>(bytes[3]) << 24U);
}
}

AuthClientEventPublisher::AuthClientEventPublisher(
    std::shared_ptr<CMsgQueue<AuthClientEvent>> events)
    : m_Events(std::move(events))
{
}

void AuthClientEventPublisher::PublishMessage(std::unique_ptr<CMessage> message) const
{
    if (!m_Events || !message) {
        return;
    }
    auto event = std::make_unique<AuthClientEvent>();
    event->payload = std::move(message);
    static_cast<void>(m_Events->PushMessage(std::move(event)));
}

void AuthClientEventPublisher::PublishReconnected(
    std::shared_ptr<CMyNetClientAuth> client) const
{
    if (!m_Events || !client) {
        return;
    }
    auto event = std::make_unique<AuthClientEvent>();
    event->payload = AuthClientReconnected{std::move(client)};
    static_cast<void>(m_Events->PushMessage(std::move(event)));
}

AuthClientEventPublisher::operator bool() const noexcept
{
    return static_cast<bool>(m_Events);
}

CMyNetClientAuth::CMyNetClientAuth(asio::any_io_executor executor)
    : m_Socket(std::move(executor)),
      m_Events(std::make_shared<CMsgQueue<AuthClientEvent>>())
{
    m_ReceiveBuffer.reserve(kClientInitialReceiveCapacity);
}

CMyNetClientAuth::~CMyNetClientAuth()
{
    asio::error_code ignored;
    if (m_Socket.is_open()) {
        m_Socket.shutdown(asio::ip::tcp::socket::shutdown_both, ignored);
        m_Socket.close(ignored);
    }
}

asio::awaitable<ClientConnectResult>
CMyNetClientAuth::Connect(const asio::ip::tcp::endpoint& remote,
                          std::optional<asio::ip::tcp::endpoint> local)
{
    if (m_Socket.is_open()) {
        asio::error_code ignored;
        m_Socket.close(ignored);
    }

    asio::error_code openError;
    m_Socket.open(asio::ip::tcp::v4(), openError);
    if (openError) {
        co_return ClientConnectResult{ClientConnectStatus::IoError, openError};
    }

    if (local) {
        asio::error_code bindError;
        m_Socket.bind(*local, bindError);
        if (bindError) {
            asio::error_code ignored;
            m_Socket.close(ignored);
            co_return ClientConnectResult{ClientConnectStatus::IoError, bindError};
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

std::int32_t CMyNetClientAuth::Close()
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

void CMyNetClientAuth::HandleTransportClose()
{
    static_cast<void>(Close());
    PublishMessage(std::make_unique<CMessage>(kCloseMessageType));
}

bool CMyNetClientAuth::IsConnected() const noexcept
{
    return m_Connected;
}

std::optional<asio::ip::tcp::endpoint> CMyNetClientAuth::ConnectedEndpoint() const noexcept
{
    return m_ConnectedEndpoint;
}

void CMyNetClientAuth::EnableControlSend() noexcept
{
    m_ControlSend = true;
}

ClientSendQueue& CMyNetClientAuth::SendQueue() noexcept
{
    return m_SendQueue;
}

const ClientSendQueue& CMyNetClientAuth::SendQueue() const noexcept
{
    return m_SendQueue;
}

asio::awaitable<ClientFlushResult> CMyNetClientAuth::FlushOutgoing()
{
    if (!m_Connected || !m_ControlSend) {
        co_return ClientFlushResult{ClientFlushStatus::IoError,
                                    0,
                                    0,
                                    std::make_error_code(std::errc::not_connected)};
    }
    co_return co_await m_SendQueue.Flush(m_Socket);
}

asio::awaitable<AuthClientReadResult> CMyNetClientAuth::ReadOnce()
{
    if (!m_Connected) {
        co_return AuthClientReadResult{
            .ioError = std::make_error_code(std::errc::not_connected),
        };
    }

    std::array<std::uint8_t, kClientMaxReceiveChunk> buffer{};
    asio::error_code error;
    const std::size_t received = co_await m_Socket.async_read_some(
        asio::buffer(buffer), asio::redirect_error(asio::use_awaitable, error));
    if (error) {
        co_return AuthClientReadResult{.ioError = error};
    }
    if (received == 0U) {
        HandleTransportClose();
        co_return AuthClientReadResult{.closed = true};
    }

    std::size_t accepted = 0;
    auto receiveError = AcceptReceivedBytes(
        std::span<const std::uint8_t>(buffer.data(), received), accepted);
    co_return AuthClientReadResult{
        .messages = accepted,
        .receiveError = std::move(receiveError),
    };
}

std::optional<AuthClientReceiveError>
CMyNetClientAuth::AcceptReceivedBytes(std::span<const std::uint8_t> received,
                                      std::size_t& acceptedMessages)
{
    acceptedMessages = 0;
    if (received.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) ||
        m_ReceiveBuffer.size() >
            static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) - received.size()) {
        return AuthClientReceiveError{
            .kind = AuthClientReceiveErrorKind::PendingSizeOverflowReactionUnknown};
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
            return AuthClientReceiveError{
                .kind = AuthClientReceiveErrorKind::LengthChecksumMismatch,
                .declared = declared,
                .expected = expectedLength,
                .actual = actualLength};
        }

        if (std::bit_cast<std::int32_t>(declared) < 0) {
            DiscardPending();
            return AuthClientReceiveError{
                .kind = AuthClientReceiveErrorKind::SignedFrameLengthReactionUnknown,
                .declared = declared};
        }

        const std::size_t frameLength = static_cast<std::size_t>(declared);
        if (frame.size() < frameLength) {
            break;
        }
        if (frameLength < kMinimumServerFrame) {
            DiscardPending();
            return AuthClientReceiveError{
                .kind = AuthClientReceiveErrorKind::ShortFrameReactionUnknown,
                .declared = declared};
        }

        CreateMessageResult created = CMessage::CreateMessageWithoutRLE(
            frame.subspan(kServerEnvelopeSize, frameLength - kServerEnvelopeSize));
        if (!created) {
            DiscardPending();
            return AuthClientReceiveError{
                .kind = AuthClientReceiveErrorKind::MessageCreateFailed,
                .declared = declared,
                .messageError = created.error};
        }

        const std::uint32_t expectedContent = ReadU32(frame.subspan<8, 4>());
        const std::uint32_t actualContent = DataCrc32(created.message->WireBytes());
        if (actualContent != expectedContent) {
            const std::int32_t type = created.message->MessageType();
            DiscardPending();
            return AuthClientReceiveError{
                .kind = AuthClientReceiveErrorKind::ContentChecksumMismatch,
                .declared = declared,
                .expected = expectedContent,
                .actual = actualContent,
                .messageType = type};
        }

        PublishMessage(std::move(created.message));
        consumed += frameLength;
        ++acceptedMessages;
    }

    if (consumed != 0U) {
        m_ReceiveBuffer.erase(m_ReceiveBuffer.begin(),
                              m_ReceiveBuffer.begin() + static_cast<std::ptrdiff_t>(consumed));
    }
    if (m_ReceiveBuffer.capacity() > kClientInitialReceiveCapacity &&
        m_ReceiveBuffer.size() <= kClientInitialReceiveCapacity) {
        std::vector<std::uint8_t> compact;
        compact.reserve(kClientInitialReceiveCapacity);
        compact.insert(compact.end(), m_ReceiveBuffer.begin(), m_ReceiveBuffer.end());
        m_ReceiveBuffer.swap(compact);
    }
    return std::nullopt;
}

std::int32_t CMyNetClientAuth::PendingEvents() const
{
    return m_Events->GetSize();
}

std::unique_ptr<AuthClientEvent> CMyNetClientAuth::PopEvent()
{
    return m_Events->PopMessage();
}

void CMyNetClientAuth::PublishReconnected(std::shared_ptr<CMyNetClientAuth> client)
{
    EventPublisher().PublishReconnected(std::move(client));
}

AuthClientEventPublisher CMyNetClientAuth::EventPublisher() const
{
    return AuthClientEventPublisher(m_Events);
}

std::size_t CMyNetClientAuth::PendingBytes() const noexcept
{
    return m_ReceiveBuffer.size();
}

void CMyNetClientAuth::PublishMessage(std::unique_ptr<CMessage> message)
{
    EventPublisher().PublishMessage(std::move(message));
}

void CMyNetClientAuth::DiscardPending() noexcept
{
    m_ReceiveBuffer.clear();
}
}
