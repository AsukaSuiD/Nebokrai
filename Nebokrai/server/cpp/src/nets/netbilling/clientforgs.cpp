#include "clientforgs.h"

#include "../../public/crc32static.h"
#include "../mysocket.h"

#include <bit>
#include <array>
#include <string>

namespace BillingNet
{
namespace
{
constexpr std::size_t kInitialReceiveCapacity = 0xA0'0000;
constexpr std::size_t kServerEnvelopeSize = 12;
constexpr std::int32_t kGameServerDisconnected = 0x0010'EF01;

std::uint32_t ReadU32(std::span<const std::uint8_t> bytes) noexcept
{
    return static_cast<std::uint32_t>(bytes[0]) |
           (static_cast<std::uint32_t>(bytes[1]) << 8U) |
           (static_cast<std::uint32_t>(bytes[2]) << 16U) |
           (static_cast<std::uint32_t>(bytes[3]) << 24U);
}

std::string FormatIPv4(std::uint32_t address)
{
    const auto bytes = std::bit_cast<std::array<std::uint8_t, 4>>(address);
    return std::to_string(bytes[0]) + "." + std::to_string(bytes[1]) + "." +
           std::to_string(bytes[2]) + "." + std::to_string(bytes[3]);
}
}

CClientForGS::CClientForGS(std::int32_t socketId,
                           std::uint32_t peerIPv4,
                           std::uint32_t nowMs)
    : CServerClient(socketId, peerIPv4, nowMs, kInitialReceiveCapacity)
{
}

void CClientForGS::OnClose(CMsgQueue<CMessage>& messages)
{
    const ServerClientMessageContext context = MessageContext();
    auto message = std::make_unique<CMessage>(kGameServerDisconnected);
    message->Base().Add(context.mapId);
    message->Base().Add(FormatIPv4(context.peerIPv4).c_str());
    message->Base().Add(static_cast<std::int32_t>(kDefaultSocketPort));
    static_cast<void>(messages.PushMessage(std::move(message)));
    MarkClosing();
}

BillingReceiveResult CClientForGS::OnReceive(CMsgQueue<CMessage>& messages)
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
                    BillingReceiveError{
                        .kind = BillingReceiveErrorKind::LengthChecksumMismatch,
                        .declared = declared,
                        .expected = expectedLength,
                        .actual = actualLength}};
        }

        if (std::bit_cast<std::int32_t>(declared) < 0) {
            return {produced,
                    PendingReceiveBytes(),
                    BillingReceiveError{
                        .kind = BillingReceiveErrorKind::SignedFrameLengthReactionUnknown,
                        .declared = declared}};
        }

        const std::size_t frameLength = static_cast<std::size_t>(declared);
        if (frame.size() < frameLength) {
            break;
        }
        if (frameLength < kServerEnvelopeSize) {
            return {produced,
                    PendingReceiveBytes(),
                    BillingReceiveError{
                        .kind = BillingReceiveErrorKind::ShortFrameReactionUnknown,
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
                    BillingReceiveError{
                        .kind = BillingReceiveErrorKind::MessageCreateFailed,
                        .declared = declared,
                        .messageError = created.error}};
        }

        const std::uint32_t expectedContent = ReadU32(frame.subspan<8, 4>());
        const std::uint32_t actualContent = DataCrc32(created.message->WireBytes());
        if (actualContent != expectedContent) {
            const std::int32_t messageType = created.message->MessageType();
            DiscardReceiveData();
            return {produced,
                    0,
                    BillingReceiveError{
                        .kind = BillingReceiveErrorKind::ContentChecksumMismatch,
                        .declared = declared,
                        .expected = expectedContent,
                        .actual = actualContent,
                        .messageType = messageType}};
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
