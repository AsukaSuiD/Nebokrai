#include "mynetserverclient_world.h"

#include "../../public/crc32static.h"

#include <bit>

namespace LoginNet
{
namespace
{
constexpr std::size_t kWorldInitialReceiveCapacity = 0xA0'0000;
constexpr std::size_t kServerEnvelopeSize = 12;
constexpr std::int32_t kWorldDisconnected = 0x0000FF01;

std::uint32_t ReadU32(std::span<const std::uint8_t> bytes) noexcept
{
    return static_cast<std::uint32_t>(bytes[0]) |
           (static_cast<std::uint32_t>(bytes[1]) << 8U) |
           (static_cast<std::uint32_t>(bytes[2]) << 16U) |
           (static_cast<std::uint32_t>(bytes[3]) << 24U);
}
}

CMyNetServerClientWorld::CMyNetServerClientWorld(std::int32_t socketId,
                                                 std::uint32_t peerIPv4,
                                                 std::uint32_t nowMs)
    : CMyServerClient(socketId, peerIPv4, nowMs, kWorldInitialReceiveCapacity)
{
}

void CMyNetServerClientWorld::OnClose(CMsgQueue<CMessage>& messages)
{
    auto message = std::make_unique<CMessage>(kWorldDisconnected);
    message->ApplyMapID(MessageContext().mapId);
    static_cast<void>(messages.PushMessage(std::move(message)));
    MarkClosing();
}

WorldReceiveResult CMyNetServerClientWorld::OnReceive(CMsgQueue<CMessage>& messages)
{
    std::int32_t produced = 0;

    while (PendingReceiveBytes() >= kServerEnvelopeSize) {
        const auto frame = ReceiveBytes();
        const std::uint32_t declared = ReadU32(frame.first<4>());
        const std::uint32_t expectedLength = ReadU32(frame.subspan<4, 4>());
        const std::uint32_t actualLength = DataCrc32(frame.first<4>());
        if (actualLength != expectedLength) {
            DiscardReceiveData();
            return {produced,
                    0,
                    WorldReceiveError{
                        .kind = WorldReceiveErrorKind::LengthChecksumMismatch,
                        .declared = declared,
                        .expected = expectedLength,
                        .actual = actualLength}};
        }

        if (std::bit_cast<std::int32_t>(declared) < 0) {
            return {produced,
                    PendingReceiveBytes(),
                    WorldReceiveError{
                        .kind = WorldReceiveErrorKind::SignedFrameLengthReactionUnknown,
                        .declared = declared}};
        }

        const std::size_t frameLength = static_cast<std::size_t>(declared);
        if (frame.size() < frameLength) {
            break;
        }
        if (frameLength < kServerEnvelopeSize) {
            return {produced,
                    PendingReceiveBytes(),
                    WorldReceiveError{
                        .kind = WorldReceiveErrorKind::ShortFrameReactionUnknown,
                        .declared = declared}};
        }

        CreateMessageResult created = CMessage::CreateMessageWithoutRLE(
            frame.subspan(kServerEnvelopeSize, frameLength - kServerEnvelopeSize));
        if (!created) {
            if (created.error && created.error->kind == CreateMessageErrorKind::EmptyInput) {
                DiscardReceiveData();
            }
            return {produced,
                    PendingReceiveBytes(),
                    WorldReceiveError{
                        .kind = WorldReceiveErrorKind::MessageCreateFailed,
                        .declared = declared,
                        .messageError = created.error}};
        }

        const std::uint32_t expectedContent = ReadU32(frame.subspan<8, 4>());
        const std::uint32_t actualContent = DataCrc32(created.message->WireBytes());
        if (actualContent != expectedContent) {
            const std::int32_t type = created.message->MessageType();
            DiscardReceiveData();
            return {produced,
                    0,
                    WorldReceiveError{
                        .kind = WorldReceiveErrorKind::ContentChecksumMismatch,
                        .declared = declared,
                        .expected = expectedContent,
                        .actual = actualContent,
                        .messageType = type}};
        }

        created.message->ApplyClientContext(MessageContext());
        static_cast<void>(messages.PushMessage(std::move(created.message)));
        produced = std::bit_cast<std::int32_t>(
            std::bit_cast<std::uint32_t>(produced) + std::uint32_t{1});
        static_cast<void>(ConsumeReceivePrefix(frameLength));
    }

    return {produced, PendingReceiveBytes(), std::nullopt};
}
}
