#include "mynetserverclient_auth.h"

#include "../../public/crc32static.h"

#include <bit>
#include <limits>

namespace
{
constexpr std::size_t kAuthInitialReceiveCapacity = 0xA0'0000;
constexpr std::size_t kServerEnvelopeSize = 12;
constexpr std::size_t kMessageHeaderSize = 16;
constexpr std::int32_t kLoginServerConnected = 0x000CF401;
constexpr std::int32_t kLoginServerDisconnected = 0x000CF402;

std::uint32_t ReadU32(std::span<const std::uint8_t> bytes) noexcept
{
    return static_cast<std::uint32_t>(bytes[0]) |
           (static_cast<std::uint32_t>(bytes[1]) << 8U) |
           (static_cast<std::uint32_t>(bytes[2]) << 16U) |
           (static_cast<std::uint32_t>(bytes[3]) << 24U);
}
}

CMyNetServerClientAuth::CMyNetServerClientAuth(std::int32_t socketId,
                                               std::uint32_t peerIPv4,
                                               std::uint32_t nowMs)
    : CServerClient(socketId, peerIPv4, nowMs, kAuthInitialReceiveCapacity)
{
}

void CMyNetServerClientAuth::OnAccept(CMsgQueue<CMessage>& messages)
{
    MarkOpen();
    auto message = std::make_unique<CMessage>(kLoginServerConnected);
    const auto context = MessageContext();
    message->ApplySocketContext(context.socketId, context.peerIPv4);
    static_cast<void>(messages.PushMessage(std::move(message)));
}

void CMyNetServerClientAuth::OnClose(CMsgQueue<CMessage>& messages)
{
    auto message = std::make_unique<CMessage>(kLoginServerDisconnected);
    const auto context = MessageContext();
    message->ApplySocketContext(context.socketId, context.peerIPv4);
    static_cast<void>(messages.PushMessage(std::move(message)));
    MarkClosing();
}

AuthReceiveResult CMyNetServerClientAuth::OnReceive(std::uint32_t recvTimeMs,
                                                     CMsgQueue<CMessage>& messages)
{
    std::int32_t produced = 0;

    while (PendingReceiveBytes() >= kServerEnvelopeSize) {
        const auto frame = ReceiveBytes();
        const std::uint32_t totalLength = ReadU32(frame.first<4>());
        const std::uint32_t lengthChecksum = ReadU32(frame.subspan<4, 4>());
        if (DataCrc32(frame.first<4>()) != lengthChecksum) {
            DiscardReceiveData();
            return {AuthReceiveStatus::LengthChecksumMismatch,
                    produced,
                    0,
                    totalLength,
                    std::nullopt};
        }

        const std::size_t totalLengthSize = static_cast<std::size_t>(totalLength);
        constexpr std::size_t minimumLength = kServerEnvelopeSize + kMessageHeaderSize;
        if (totalLengthSize < minimumLength ||
            totalLength > static_cast<std::uint32_t>(std::numeric_limits<std::int32_t>::max())) {
            return {AuthReceiveStatus::EnvelopeLengthReactionUnknown,
                    produced,
                    PendingReceiveBytes(),
                    totalLength,
                    std::nullopt};
        }
        if (frame.size() < totalLengthSize) {
            break;
        }

        const std::uint32_t expectedMessageChecksum = ReadU32(frame.subspan<8, 4>());
        const auto wire = frame.subspan(kServerEnvelopeSize,
                                        totalLengthSize - kServerEnvelopeSize);
        AuthCreateMessageResult created = CMessage::CreateMessageWithoutRLE(wire, recvTimeMs);
        if (!created) {
            return {AuthReceiveStatus::CreateMessageFailed,
                    produced,
                    PendingReceiveBytes(),
                    totalLength,
                    created.error};
        }
        if (DataCrc32(created.message->WireBytes()) != expectedMessageChecksum) {
            DiscardReceiveData();
            return {AuthReceiveStatus::MessageChecksumMismatch,
                    produced,
                    0,
                    totalLength,
                    std::nullopt};
        }

        created.message->ApplyClientContext(MessageContext());
        static_cast<void>(messages.PushMessage(std::move(created.message)));
        produced = std::bit_cast<std::int32_t>(
            std::bit_cast<std::uint32_t>(produced) + std::uint32_t{1});
        static_cast<void>(ConsumeReceivePrefix(totalLengthSize));
    }

    return {AuthReceiveStatus::Ok,
            produced,
            PendingReceiveBytes(),
            0,
            std::nullopt};
}
