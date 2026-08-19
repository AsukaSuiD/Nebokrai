#include "message.h"

#include "../../public/crc32static.h"

#include <algorithm>
#include <array>
#include <bit>
#include <limits>
#include <mutex>
#include <utility>

namespace LoginNet
{
namespace
{
constexpr std::size_t kClientEnvelopeSize = 4;
constexpr std::size_t kServerEnvelopeSize = 12;
constexpr std::size_t kSmallRleInputLimit = 0x20'001;
constexpr std::size_t kSmallRleOutputCapacity = 0x10'0000;
constexpr std::uint32_t kMessageFamilyMask = 0xFFFF'FF00U;

std::mutex g_AuthSendSerializer;

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
                   std::span<const std::uint8_t> payload)
    : CBaseMessage(header, payload)
{
}

CreateMessageResult CMessage::Create(std::span<const std::uint8_t> compressed)
{
    if (compressed.empty()) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::EmptyInput, std::nullopt}};
    }
    if (compressed.size() > std::numeric_limits<std::uint32_t>::max()) {
        return {nullptr,
                CreateMessageError{CreateMessageErrorKind::InputOutsideLegacyRange,
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
        return {nullptr,
                CreateMessageError{CreateMessageErrorKind::RleDecode, decoded.error}};
    }
    return CreateMessageWithoutRLE(decoded.bytes);
}

CreateMessageResult CMessage::CreateMessageWithoutRLE(std::span<const std::uint8_t> wire)
{
    if (wire.empty()) {
        return {nullptr, CreateMessageError{CreateMessageErrorKind::EmptyInput, std::nullopt}};
    }
    if (wire.size() > std::numeric_limits<std::uint32_t>::max()) {
        return {nullptr,
                CreateMessageError{CreateMessageErrorKind::InputOutsideLegacyRange,
                                   std::nullopt}};
    }
    if (wire.size() < kHeaderSize) {
        return {nullptr,
                CreateMessageError{CreateMessageErrorKind::HeaderTooShortReactionUnknown,
                                   std::nullopt}};
    }

    std::array<std::uint8_t, kHeaderSize> header{};
    std::copy_n(wire.begin(), kHeaderSize, header.begin());
    return {
        std::unique_ptr<CMessage>(new CMessage(
            std::span<const std::uint8_t, kHeaderSize>(header),
            wire.subspan(kHeaderSize))),
        std::nullopt,
    };
}

std::int32_t CMessage::MessageType() const noexcept
{
    return std::bit_cast<std::int32_t>(GetType());
}

void CMessage::SetMessageType(std::int32_t messageType) noexcept
{
    SetType(std::bit_cast<std::uint32_t>(messageType));
}

CBaseMessage& CMessage::Base() noexcept
{
    return *this;
}

const CBaseMessage& CMessage::Base() const noexcept
{
    return *this;
}

std::vector<std::uint8_t> CMessage::GetString()
{
    return GetCStringBytes();
}

std::span<const std::uint8_t> CMessage::WireBytes() const noexcept
{
    return {Data(), DataSize()};
}

void CMessage::ApplyClientContext(const ServerClientMessageContext& context)
{
    m_SocketID = context.socketId;
    m_MapID = context.mapId;
    m_Cdkey.assign(context.mapName.begin(), context.mapName.end());
    m_IP = context.peerIPv4;
}

void CMessage::ApplyMapID(std::int32_t mapId) noexcept
{
    m_MapID = mapId;
}

std::int32_t CMessage::SocketID() const noexcept
{
    return m_SocketID;
}

std::int32_t CMessage::MapID() const noexcept
{
    return m_MapID;
}

std::span<const std::uint8_t> CMessage::Cdkey() const noexcept
{
    return m_Cdkey;
}

std::uint32_t CMessage::IP() const noexcept
{
    return m_IP;
}

std::variant<std::vector<std::uint8_t>, SendMessageError>
CMessage::ClientEnvelope() const
{
    const auto compressed = CBaseMessage::DoRLE(WireBytes());
    if (!compressed) {
        return SendMessageError::RleEmptyInputReactionUnknown;
    }
    if (compressed->size() >
        static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) -
            kClientEnvelopeSize) {
        return SendMessageError::LengthOutsideLegacyRange;
    }

    const std::int32_t totalLength =
        static_cast<std::int32_t>(compressed->size() + kClientEnvelopeSize);
    std::vector<std::uint8_t> envelope;
    envelope.reserve(static_cast<std::size_t>(totalLength));
    AppendU32(envelope, std::bit_cast<std::uint32_t>(totalLength));
    envelope.insert(envelope.end(), compressed->begin(), compressed->end());
    return envelope;
}

std::variant<std::vector<std::uint8_t>, SendMessageError>
CMessage::ServerEnvelope() const
{
    if (DataSize() >
        static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) -
            kServerEnvelopeSize) {
        return SendMessageError::LengthOutsideLegacyRange;
    }

    const std::int32_t totalLength =
        static_cast<std::int32_t>(DataSize() + kServerEnvelopeSize);
    const std::uint32_t totalWord = std::bit_cast<std::uint32_t>(totalLength);
    std::array<std::uint8_t, 4> lengthBytes{
        static_cast<std::uint8_t>(totalWord),
        static_cast<std::uint8_t>(totalWord >> 8U),
        static_cast<std::uint8_t>(totalWord >> 16U),
        static_cast<std::uint8_t>(totalWord >> 24U),
    };

    std::vector<std::uint8_t> envelope;
    envelope.reserve(static_cast<std::size_t>(totalLength));
    envelope.insert(envelope.end(), lengthBytes.begin(), lengthBytes.end());
    AppendU32(envelope, DataCrc32(lengthBytes));
    AppendU32(envelope, DataCrc32(WireBytes()));
    envelope.insert(envelope.end(), Data(), Data() + DataSize());
    return envelope;
}

std::variant<std::int32_t, SendMessageError>
CMessage::SendToClientSocket(const ServerCommandHandle& sender,
                             std::int32_t socketId) const
{
    auto envelope = ClientEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) {
        return *error;
    }
    return sender.SendBySocketID(socketId, std::get<std::vector<std::uint8_t>>(envelope));
}

std::variant<std::int32_t, SendMessageError>
CMessage::SendToClientCdkey(const ServerCommandHandle& sender,
                            std::span<const std::uint8_t> cdkey) const
{
    auto envelope = ClientEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) {
        return *error;
    }
    return sender.SendByMapName(cdkey, std::get<std::vector<std::uint8_t>>(envelope));
}

std::variant<std::int32_t, SendMessageError>
CMessage::SendToWorldSocket(const ServerCommandHandle& sender,
                            std::int32_t socketId) const
{
    auto envelope = ServerEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) {
        return *error;
    }
    return sender.SendBySocketID(socketId, std::get<std::vector<std::uint8_t>>(envelope));
}

std::variant<std::int32_t, SendMessageError>
CMessage::SendToWorldMap(const ServerCommandHandle& sender,
                         std::int32_t mapId) const
{
    auto envelope = ServerEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) {
        return *error;
    }
    return sender.SendByMapID(mapId, std::get<std::vector<std::uint8_t>>(envelope));
}

std::variant<std::int32_t, SendMessageError>
CMessage::SendAllWorld(const ServerCommandHandle& sender) const
{
    auto envelope = ServerEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) {
        return *error;
    }
    return sender.SendAll(std::get<std::vector<std::uint8_t>>(envelope));
}

std::variant<std::int32_t, SendMessageError>
CMessage::SendToAuth(ClientSendQueue& sender) const
{
    std::lock_guard<std::mutex> lock(g_AuthSendSerializer);
    auto envelope = ServerEnvelope();
    if (const auto* error = std::get_if<SendMessageError>(&envelope)) {
        return *error;
    }
    return sender.SendToServer(std::get<std::vector<std::uint8_t>>(envelope), false, 0);
}

std::int32_t CMessage::Run(ILoginMessageHandlers& handlers)
{
    const std::uint32_t opcode = GetType();
    if (0x000C'F300U < opcode && opcode < 0x000D'F1FFU) {
        handlers.OnAuth(*this);
        return 1;
    }

    switch (opcode & kMessageFamilyMask) {
    case 0x0002'0000U:
        handlers.OnGM(*this);
        break;
    case 0x0002'0100U:
        handlers.OnGMA(*this);
        break;
    case 0x0001'FF00U:
    case 0x0002'FD00U:
    case 0x0001'0000U:
        handlers.OnLog(*this);
        break;
    case 0x0000'FF00U:
    case 0x0001'FE00U:
        handlers.OnServer(*this);
        break;
    default:
        break;
    }
    return 1;
}
}
