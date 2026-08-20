#include "myserverclient.h"

#include "../../public/crc32static.h"

#include <bit>

namespace WorldNet
{
namespace
{
constexpr std::size_t kInitialReceiveCapacity = 0x140'0000;
constexpr std::size_t kEnvelopeSize = 12;
constexpr std::int32_t kGameServerDisconnected = 0x0003'FC02;

std::uint32_t ReadU32(std::span<const std::uint8_t> bytes) noexcept
{
    return static_cast<std::uint32_t>(bytes[0]) |
           (static_cast<std::uint32_t>(bytes[1]) << 8U) |
           (static_cast<std::uint32_t>(bytes[2]) << 16U) |
           (static_cast<std::uint32_t>(bytes[3]) << 24U);
}
}

CMyServerClient::CMyServerClient(std::int32_t socketId,
                                 std::uint32_t peerIPv4,
                                 std::uint32_t nowMs)
    : CServerClient(socketId, peerIPv4, nowMs, kInitialReceiveCapacity)
{
}

void CMyServerClient::OnClose(CMsgQueue<CMessage>& messages)
{
    auto message = std::make_unique<CMessage>(kGameServerDisconnected);
    message->Base().Add(MessageContext().mapId);
    static_cast<void>(messages.PushMessage(std::move(message)));
    MarkClosing();
}

GameServerReceiveResult CMyServerClient::OnReceive(CMsgQueue<CMessage>& messages,
                                                   std::uint32_t receiveTimeMs)
{
    std::int32_t produced = 0;
    while (PendingReceiveBytes() >= kEnvelopeSize) {
        const auto frame = ReceiveBytes();
        const std::uint32_t declared = ReadU32(frame.first<4>());
        const std::uint32_t expectedLength = ReadU32(frame.subspan<4, 4>());
        const std::uint32_t actualLength = DataCrc32(frame.first<4>());
        if (actualLength != expectedLength) {
            DiscardReceiveData();
            return {produced,
                    0,
                    GameServerReceiveError{
                        .kind = GameServerReceiveErrorKind::LengthChecksumMismatch,
                        .declared = declared,
                        .expected = expectedLength,
                        .actual = actualLength}};
        }
        if (std::bit_cast<std::int32_t>(declared) < 0) {
            return {produced,
                    PendingReceiveBytes(),
                    GameServerReceiveError{
                        .kind = GameServerReceiveErrorKind::SignedFrameLengthReactionUnknown,
                        .declared = declared}};
        }

        const std::size_t frameLength = static_cast<std::size_t>(declared);
        if (frame.size() < frameLength) break;
        if (frameLength < kEnvelopeSize) {
            return {produced,
                    PendingReceiveBytes(),
                    GameServerReceiveError{
                        .kind = GameServerReceiveErrorKind::ShortFrameReactionUnknown,
                        .declared = declared}};
        }

        CreateMessageResult created = CMessage::CreateMessageWithoutRLE(
            frame.subspan(kEnvelopeSize, frameLength - kEnvelopeSize), receiveTimeMs);
        if (!created) {
            if (created.error && created.error->kind == CreateMessageErrorKind::EmptyInput) {
                DiscardReceiveData();
            }
            return {produced,
                    PendingReceiveBytes(),
                    GameServerReceiveError{
                        .kind = GameServerReceiveErrorKind::MessageCreateFailed,
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
                    GameServerReceiveError{
                        .kind = GameServerReceiveErrorKind::ContentChecksumMismatch,
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
