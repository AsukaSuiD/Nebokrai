#include "mynetserverclient_client.h"

#include "../../public/crc32static.h"

#include <bit>

namespace LoginNet
{
namespace
{
constexpr std::size_t kClientInitialReceiveCapacity = 0x5000;
constexpr std::size_t kClientEnvelopeSize = 12;
constexpr std::uint32_t kClientMessageMin = 0x0002FD01U;
constexpr std::uint32_t kClientMessageMax = 0x0003FBFFU;
constexpr std::int32_t kClientDisconnected = 0x00010001;

std::uint32_t ReadU32(std::span<const std::uint8_t> bytes) noexcept
{
    return static_cast<std::uint32_t>(bytes[0]) |
           (static_cast<std::uint32_t>(bytes[1]) << 8U) |
           (static_cast<std::uint32_t>(bytes[2]) << 16U) |
           (static_cast<std::uint32_t>(bytes[3]) << 24U);
}

bool IsProvenNullCreate(const CreateMessageError& error) noexcept
{
    if (error.kind == CreateMessageErrorKind::EmptyInput) {
        return true;
    }
    if (error.kind != CreateMessageErrorKind::RleDecode || !error.rleError) {
        return false;
    }
    return *error.rleError == RleDecodeError::EmptyInput ||
           *error.rleError == RleDecodeError::OutputCapacityReached;
}
}

bool ClientReceiveError::RequiresForbidAndQuit() const noexcept
{
    return kind == ClientReceiveErrorKind::MessageLengthExceeded ||
           kind == ClientReceiveErrorKind::LengthChecksumMismatch ||
           kind == ClientReceiveErrorKind::ContentChecksumMismatch ||
           kind == ClientReceiveErrorKind::OpcodeOutsideAllowedRange;
}

CMyNetServerClientClient::CMyNetServerClientClient(std::int32_t socketId,
                                                   std::uint32_t peerIPv4,
                                                   std::uint32_t nowMs)
    : CMyServerClient(socketId, peerIPv4, nowMs, kClientInitialReceiveCapacity)
{
}

bool CMyNetServerClientClient::OnClose(CMsgQueue<CMessage>& messages)
{
    const auto context = MessageContext();
    if (context.mapName.empty()) {
        return false;
    }

    auto message = std::make_unique<CMessage>(kClientDisconnected);
    message->Base().Add(context.mapName.data(),
                        static_cast<std::int32_t>(context.mapName.size()));
    message->Base().Add(std::uint8_t{0});
    static_cast<void>(messages.PushMessage(std::move(message)));
    MarkClosing();
    return true;
}

ClientReceiveResult CMyNetServerClientClient::OnReceive(
    ClientReceiveSettings settings,
    CMsgQueue<CMessage>& messages)
{
    std::int32_t produced = 0;

    while (PendingReceiveBytes() >= kClientEnvelopeSize) {
        const auto frame = ReceiveBytes();
        const std::uint32_t declared = ReadU32(frame.first<4>());

        if (settings.checkLengthCrc) {
            if (std::bit_cast<std::int32_t>(settings.maximumMessageLength) <
                std::bit_cast<std::int32_t>(declared)) {
                return {produced,
                        PendingReceiveBytes(),
                        ClientReceiveError{
                            .kind = ClientReceiveErrorKind::MessageLengthExceeded,
                            .declared = declared,
                            .permitted = settings.maximumMessageLength}};
            }

            const std::uint32_t expected = ReadU32(frame.subspan<4, 4>());
            const std::uint32_t actual = DataCrc32(frame.first<4>());
            if (actual != expected) {
                return {produced,
                        PendingReceiveBytes(),
                        ClientReceiveError{
                            .kind = ClientReceiveErrorKind::LengthChecksumMismatch,
                            .declared = declared,
                            .expected = expected,
                            .actual = actual}};
            }
        }

        if (std::bit_cast<std::int32_t>(declared) < 0) {
            return {produced,
                    PendingReceiveBytes(),
                    ClientReceiveError{
                        .kind = ClientReceiveErrorKind::SignedFrameLengthReactionUnknown,
                        .declared = declared}};
        }

        const std::size_t frameLength = static_cast<std::size_t>(declared);
        if (frame.size() < frameLength) {
            break;
        }
        if (frameLength < kClientEnvelopeSize) {
            return {produced,
                    PendingReceiveBytes(),
                    ClientReceiveError{
                        .kind = ClientReceiveErrorKind::ShortFrameReactionUnknown,
                        .declared = declared}};
        }

        const auto compressed = frame.subspan(kClientEnvelopeSize,
                                               frameLength - kClientEnvelopeSize);
        if (settings.checkContentCrc) {
            const std::uint32_t expected = ReadU32(frame.subspan<8, 4>());
            const std::uint32_t actual = DataCrc32(compressed);
            if (actual != expected) {
                return {produced,
                        PendingReceiveBytes(),
                        ClientReceiveError{
                            .kind = ClientReceiveErrorKind::ContentChecksumMismatch,
                            .declared = declared,
                            .expected = expected,
                            .actual = actual}};
            }
        }

        CreateMessageResult created = CMessage::Create(compressed);
        if (!created) {
            if (created.error && IsProvenNullCreate(*created.error)) {
                DiscardReceiveData();
            }
            return {produced,
                    PendingReceiveBytes(),
                    ClientReceiveError{
                        .kind = ClientReceiveErrorKind::MessageCreateFailed,
                        .declared = declared,
                        .messageError = created.error}};
        }

        const std::uint32_t opcode = created.message->GetType();
        if (opcode < kClientMessageMin || opcode > kClientMessageMax) {
            return {produced,
                    PendingReceiveBytes(),
                    ClientReceiveError{
                        .kind = ClientReceiveErrorKind::OpcodeOutsideAllowedRange,
                        .declared = declared,
                        .opcode = opcode}};
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
