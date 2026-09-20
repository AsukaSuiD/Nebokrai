#include "message.h"

#include "../../public/crc32static.h"

#include <algorithm>
#include <array>
#include <bit>
#include <limits>
#include <utility>

namespace
{
constexpr std::size_t kServerEnvelopeSize = 12;
constexpr std::uint32_t kBlockedDeleteHandlerOpcode = 0x0010F101U;

void AppendU32(std::vector<std::uint8_t>& output, std::uint32_t value)
{
    output.push_back(static_cast<std::uint8_t>(value));
    output.push_back(static_cast<std::uint8_t>(value >> 8U));
    output.push_back(static_cast<std::uint8_t>(value >> 16U));
    output.push_back(static_cast<std::uint8_t>(value >> 24U));
}

std::optional<AuthMessageKind> Classify(std::uint32_t opcode) noexcept
{
    switch (opcode) {
    case 0x000CF401U: return AuthMessageKind::LoginServerConnected;
    case 0x000CF402U: return AuthMessageKind::LoginServerDisconnected;
    case 0x000CF501U: return AuthMessageKind::AuthenticateAccount;
    case 0x000CF502U: return AuthMessageKind::AuthenticateAccountExtended;
    case 0x000CF503U: return AuthMessageKind::GetLoginServerInfo;
    case 0x000CF802U: return AuthMessageKind::UpdateServerInfoResponse;
    case 0x0010F102U: return AuthMessageKind::GmKickPlayer;
    case 0x000CF801U: return AuthMessageKind::KickPlayerResponse;
    case 0x0010F103U: return AuthMessageKind::GmLockAccount;
    default: return std::nullopt;
    }
}
}

CMessage::CMessage(std::int32_t messageType)
{
    SetType(std::bit_cast<std::uint32_t>(messageType));
}

CMessage::CMessage(std::span<const std::uint8_t, kHeaderSize> header,
                   std::span<const std::uint8_t> payload,
                   std::uint32_t recvTimeMs)
    : CBaseMessage(header, payload),
      m_RecvTimeMs(recvTimeMs)
{
}

AuthCreateMessageResult
CMessage::CreateMessageWithoutRLE(std::span<const std::uint8_t> wire,
                                  std::uint32_t recvTimeMs)
{
    if (wire.empty()) {
        return {nullptr, AuthCreateMessageError::EmptyInputReactionUnknown};
    }
    if (wire.size() > std::numeric_limits<std::uint32_t>::max()) {
        return {nullptr, AuthCreateMessageError::InputOutsideLegacyRange};
    }
    if (wire.size() < kHeaderSize) {
        return {nullptr, AuthCreateMessageError::HeaderTooShortReactionUnknown};
    }

    std::array<std::uint8_t, kHeaderSize> header{};
    std::copy_n(wire.begin(), kHeaderSize, header.begin());
    return {
        std::unique_ptr<CMessage>(new CMessage(
            std::span<const std::uint8_t, kHeaderSize>(header),
            wire.subspan(kHeaderSize),
            recvTimeMs)),
        std::nullopt,
    };
}

std::int32_t CMessage::MessageType() const noexcept
{
    return std::bit_cast<std::int32_t>(GetType());
}

CBaseMessage& CMessage::Base() noexcept
{
    return *this;
}

const CBaseMessage& CMessage::Base() const noexcept
{
    return *this;
}

std::vector<std::uint8_t> CMessage::GetStr()
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

void CMessage::ApplySocketContext(std::int32_t socketId, std::uint32_t peerIPv4) noexcept
{
    m_SocketID = socketId;
    m_IP = peerIPv4;
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

std::uint32_t CMessage::RecvTimeMs() const noexcept
{
    return m_RecvTimeMs;
}

AuthEnvelopeResult CMessage::ServerEnvelope() const
{
    if (DataSize() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()) -
                         kServerEnvelopeSize) {
        return {{}, AuthSendMessageError::LengthOutsideLegacyRange};
    }

    const auto totalLength = static_cast<std::int32_t>(DataSize() + kServerEnvelopeSize);
    const auto totalWord = std::bit_cast<std::uint32_t>(totalLength);
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
    return {std::move(envelope), std::nullopt};
}

std::variant<std::int32_t, AuthSendMessageError>
CMessage::SendToLogin(const ServerCommandHandle& sender, std::int32_t socketId) const
{
    AuthEnvelopeResult envelope = ServerEnvelope();
    if (!envelope) {
        return *envelope.error;
    }
    return sender.SendBySocketID(socketId, envelope.bytes);
}

AuthDispatchStatus CMessage::Run(IAuthMessageHandler& handler,
                                 const ServerCommandHandle& sender)
{
    const std::uint32_t opcode = GetType();
    if (opcode == kBlockedDeleteHandlerOpcode) {
        return AuthDispatchStatus::DeleteHandlerAt10F101Unresolved;
    }

    const auto kind = Classify(opcode);
    if (!kind) {
        return AuthDispatchStatus::Ok;
    }
    return handler.Handle(*kind, *this, sender);
}
