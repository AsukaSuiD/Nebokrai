#include "message.h"

#include "../../public/crc32static.h"

#include <algorithm>
#include <array>
#include <bit>
#include <limits>
#include <mutex>

namespace WorldNet
{
namespace
{
constexpr std::size_t kEnvelopeSize = 12;
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
    : CBaseMessage(header, payload), m_ReceiveTimeMs(receiveTimeMs)
{
}

CreateMessageResult CMessage::Create(std::span<const std::uint8_t> compressed,
                                     std::uint32_t receiveTimeMs)
{
    if (compressed.empty()) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::EmptyInput}};
    }
    if (compressed.size() > std::numeric_limits<std::uint32_t>::max()) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::InputOutsideLegacyRange}};
    }

    std::size_t capacity = kSmallRleOutputCapacity;
    if (compressed.size() >= kSmallRleInputLimit) {
        if (compressed.size() >
            static_cast<std::size_t>(std::numeric_limits<std::uint32_t>::max()) / 8U) {
            return {nullptr,
                    CreateMessageError{
                        CreateMessageErrorKind::RleCapacityOverflowReactionUnknown}};
        }
        capacity = compressed.size() * 8U;
    }

    RleDecodeResult decoded = CBaseMessage::DecodeRLE(compressed, capacity);
    if (!decoded) {
        return {nullptr,
                CreateMessageError{CreateMessageErrorKind::RleDecode, decoded.error}};
    }
    return CreateMessageWithoutRLE(decoded.bytes, receiveTimeMs);
}

CreateMessageResult CMessage::CreateMessageWithoutRLE(
    std::span<const std::uint8_t> wire,
    std::uint32_t receiveTimeMs)
{
    if (wire.empty()) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::EmptyInput}};
    }
    if (wire.size() > std::numeric_limits<std::uint32_t>::max()) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::InputOutsideLegacyRange}};
    }
    if (wire.size() < kHeaderSize) {
        return {nullptr,
                CreateMessageError{CreateMessageErrorKind::HeaderTooShortReactionUnknown}};
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

void CMessage::ApplyClientContext(const ServerClientMessageContext& context) noexcept
{
    m_SocketID = context.socketId;
    m_MapID = context.mapId;
    m_IP = context.peerIPv4;
}

void CMessage::ApplyMapID(std::int32_t mapId) noexcept { m_MapID = mapId; }
std::int32_t CMessage::MapID() const noexcept { return m_MapID; }
std::int32_t CMessage::SocketID() const noexcept { return m_SocketID; }
std::uint32_t CMessage::PeerIPv4() const noexcept { return m_IP; }
std::uint32_t CMessage::ReceiveTimeMs() const noexcept { return m_ReceiveTimeMs; }

std::variant<std::vector<std::uint8_t>, SendMessageError>
CMessage::ServerEnvelope() const
{
    if (DataSize() >
        static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) -
            kEnvelopeSize) {
        return SendMessageError::LengthOutsideLegacyRange;
    }

    const auto totalLength = static_cast<std::int32_t>(DataSize() + kEnvelopeSize);
    const std::uint32_t lengthWord = std::bit_cast<std::uint32_t>(totalLength);
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
CMessage::SendToSocket(const ServerCommandHandle* sender, std::int32_t socketId) const
{
    if (sender == nullptr) return std::int32_t{0};
    std::lock_guard lock(g_SendSerializer);
    auto envelope = ServerEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) return *error;
    return sender->SendBySocketID(socketId, std::get<std::vector<std::uint8_t>>(envelope));
}

std::variant<std::int32_t, SendMessageError>
CMessage::SendToMapID(const ServerCommandHandle* sender, std::int32_t mapId) const
{
    if (sender == nullptr) return std::int32_t{0};
    std::lock_guard lock(g_SendSerializer);
    auto envelope = ServerEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) return *error;
    return sender->SendByMapID(mapId, std::get<std::vector<std::uint8_t>>(envelope));
}

std::variant<std::int32_t, SendMessageError>
CMessage::SendAll(const ServerCommandHandle* sender) const
{
    if (sender == nullptr) return std::int32_t{0};
    std::lock_guard lock(g_SendSerializer);
    auto envelope = ServerEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) return *error;
    return sender->SendAll(std::get<std::vector<std::uint8_t>>(envelope));
}

std::variant<std::int32_t, SendMessageError>
CMessage::Send(ClientSendQueue* sender, bool prioritized) const
{
    if (sender == nullptr) return std::int32_t{0};
    std::lock_guard lock(g_SendSerializer);
    auto envelope = ServerEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) return *error;
    return sender->SendToServer(std::get<std::vector<std::uint8_t>>(envelope), prioritized, 0);
}

std::int32_t CMessage::Run(IWorldMessageHandlers& handlers)
{
    switch (GetType() & kMessageFamilyMask) {
    case 0x0003'FC00U:
    case 0x0004'FC00U:
    case 0x0005'FA00U: handlers.OnServer(*this); break;
    case 0x0004'FB00U:
    case 0x0005'FB00U: handlers.OnLog(*this); break;
    case 0x0004'FD00U:
    case 0x0006'0400U: handlers.OnGma(*this); break;
    case 0x0005'FC00U: handlers.OnPlayer(*this); break;
    case 0x0005'FD00U: handlers.OnOther(*this); break;
    case 0x0005'FF00U: handlers.OnGm(*this); break;
    case 0x0006'0000U: handlers.OnTeam(*this); break;
    case 0x0006'0100U: handlers.OnOrganizingSystem(*this); break;
    case 0x0006'0200U:
        if (handlers.WriteLogEnabled()) handlers.OnWriteLog(*this);
        break;
    case 0x0006'0300U:
    case 0x0007'FF00U: handlers.OnCountry(*this); break;
    case 0x0006'0800U: handlers.OnServerAuction(*this); break;
    case 0x0006'0900U: handlers.OnJjcSystem(*this); break;
    case 0x0015'EB00U: handlers.OnMiscAuction(*this); break;
    default: break;
    }
    return 1;
}
}
