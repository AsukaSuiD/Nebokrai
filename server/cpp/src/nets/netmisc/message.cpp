#include "message.h"

#include "../../public/crc32static.h"

#include <algorithm>
#include <array>
#include <bit>
#include <limits>
#include <mutex>

namespace MiscNet
{
namespace
{
constexpr std::size_t kServerEnvelopeSize = 12;
constexpr std::size_t kSmallRleInputLimit = 0x20'001;
constexpr std::size_t kSmallRleOutputCapacity = 0x10'0000;
constexpr std::uint32_t kMessageFamilyMask = 0xFFFF'FF00U;
std::mutex g_SendSerializer;

void AppendU32(std::vector<std::uint8_t>& output, std::uint32_t value)
{
    output.push_back(static_cast<std::uint8_t>(value));
    output.push_back(static_cast<std::uint8_t>(value >> 8U));
    output.push_back(static_cast<std::uint8_t>(value >> 16U));
    output.push_back(static_cast<std::uint8_t>(value >> 24U));
}
}

CMessage::CMessage(std::int32_t messageType)
{
    SetType(std::bit_cast<std::uint32_t>(messageType));
}

CMessage::CMessage(std::span<const std::uint8_t, kHeaderSize> header,
                   std::span<const std::uint8_t> payload,
                   std::uint32_t receiveTimeMs)
    : CBaseMessage(header, payload),
      m_ReceiveTimeMs(receiveTimeMs)
{
}

CreateMessageResult CMessage::Create(std::span<const std::uint8_t> compressed,
                                     std::uint32_t receiveTimeMs)
{
    if (compressed.empty()) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::EmptyInput, std::nullopt}};
    }
    if (compressed.size() > std::numeric_limits<std::uint32_t>::max()) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::InputOutsideLegacyRange,
                                            std::nullopt}};
    }

    std::size_t capacity = kSmallRleOutputCapacity;
    if (compressed.size() >= kSmallRleInputLimit) {
        if (compressed.size() >
            static_cast<std::size_t>(std::numeric_limits<std::uint32_t>::max()) / 8U) {
            return {nullptr,
                    CreateMessageError{
                        CreateMessageErrorKind::RleCapacityOverflowReactionUnknown,
                        std::nullopt}};
        }
        capacity = compressed.size() * 8U;
    }

    RleDecodeResult decoded = CBaseMessage::DecodeRLE(compressed, capacity);
    if (!decoded) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::RleDecode, decoded.error}};
    }
    return CreateMessageWithoutRLE(decoded.bytes, receiveTimeMs);
}

CreateMessageResult CMessage::CreateMessageWithoutRLE(
    std::span<const std::uint8_t> wire,
    std::uint32_t receiveTimeMs)
{
    if (wire.empty()) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::EmptyInput, std::nullopt}};
    }
    if (wire.size() > std::numeric_limits<std::uint32_t>::max()) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::InputOutsideLegacyRange,
                                            std::nullopt}};
    }
    if (wire.size() < kHeaderSize) {
        return {nullptr,
                CreateMessageError{CreateMessageErrorKind::HeaderTooShortReactionUnknown,
                                   std::nullopt}};
    }

    std::array<std::uint8_t, kHeaderSize> header{};
    std::copy_n(wire.begin(), kHeaderSize, header.begin());
    return {std::unique_ptr<CMessage>(new CMessage(
                std::span<const std::uint8_t, kHeaderSize>(header),
                wire.subspan(kHeaderSize),
                receiveTimeMs)),
            std::nullopt};
}

std::int32_t CMessage::MessageType() const noexcept
{
    return std::bit_cast<std::int32_t>(GetType());
}

CBaseMessage& CMessage::Base() noexcept { return *this; }

std::span<const std::uint8_t> CMessage::WireBytes() const noexcept
{
    return {Data(), DataSize()};
}

std::uint32_t CMessage::ReceiveTimeMs() const noexcept { return m_ReceiveTimeMs; }

std::variant<std::vector<std::uint8_t>, SendMessageError>
CMessage::ServerEnvelope() const
{
    if (DataSize() >
        static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) -
            kServerEnvelopeSize) {
        return SendMessageError::LengthOutsideLegacyRange;
    }

    const auto totalLength = static_cast<std::int32_t>(DataSize() + kServerEnvelopeSize);
    const auto lengthWord = std::bit_cast<std::uint32_t>(totalLength);
    const std::array<std::uint8_t, 4> lengthBytes{
        static_cast<std::uint8_t>(lengthWord),
        static_cast<std::uint8_t>(lengthWord >> 8U),
        static_cast<std::uint8_t>(lengthWord >> 16U),
        static_cast<std::uint8_t>(lengthWord >> 24U)};

    std::vector<std::uint8_t> envelope;
    envelope.reserve(static_cast<std::size_t>(totalLength));
    envelope.insert(envelope.end(), lengthBytes.begin(), lengthBytes.end());
    AppendU32(envelope, DataCrc32(lengthBytes));
    AppendU32(envelope, DataCrc32(WireBytes()));
    envelope.insert(envelope.end(), Data(), Data() + DataSize());
    return envelope;
}

std::variant<std::int32_t, SendMessageError>
CMessage::Send(ClientSendQueue* sender, bool prioritized) const
{
    if (sender == nullptr) {
        return std::int32_t{0};
    }
    std::lock_guard lock(g_SendSerializer);
    auto envelope = ServerEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) {
        return *error;
    }
    return sender->SendToServer(std::get<std::vector<std::uint8_t>>(envelope),
                                prioritized,
                                0);
}

std::int32_t CMessage::Run(IMiscMessageHandlers& handlers)
{
    switch (GetType() & kMessageFamilyMask) {
    case 0x0014'EC00U:
        return 1;
    case 0x0014'ED00U:
        handlers.OnWorldAuction(*this);
        return 1;
    case 0x0016'EA00U:
        handlers.OnMiscFunction(*this);
        return 1;
    default:
        handlers.OnOther(*this);
        return 0;
    }
}
}
