#pragma once

#include "../basemessage.h"
#include "../clients.h"
#include "../servers.h"

#include <cstdint>
#include <memory>
#include <optional>
#include <span>
#include <variant>
#include <vector>

/*
 * Исходный владелец: nets/networld/message.cpp / message.h.
 *
 * Источник: Nworldserver.exe + WorldServer.pdb, поздняя Rust-реконструкция и
 * C++-каркас. Подтверждены 16-байтовое внутреннее сообщение, внешний envelope
 * [длина, CRC длины, CRC сообщения, сообщение], metadata принятого GameServer
 * и маршрутизация тринадцати семейств. Общие scratch-буферы заменены
 * владеющими std::vector и mutex; wire и порядок отправки сохранены.
 */
namespace WorldNet
{
class CMessage;

enum class CreateMessageErrorKind
{
    EmptyInput,
    HeaderTooShortReactionUnknown,
    InputOutsideLegacyRange,
    RleCapacityOverflowReactionUnknown,
    RleDecode,
};

struct CreateMessageError
{
    CreateMessageErrorKind kind{CreateMessageErrorKind::EmptyInput};
    std::optional<RleDecodeError> rleError;
};

struct CreateMessageResult
{
    std::unique_ptr<CMessage> message;
    std::optional<CreateMessageError> error;
    [[nodiscard]] explicit operator bool() const noexcept { return message != nullptr; }
};

enum class SendMessageError
{
    LengthOutsideLegacyRange,
};

class IWorldMessageHandlers
{
public:
    virtual ~IWorldMessageHandlers() = default;
    [[nodiscard]] virtual bool WriteLogEnabled() const noexcept = 0;
    virtual void OnServer(CMessage& message) = 0;
    virtual void OnLog(CMessage& message) = 0;
    virtual void OnGma(CMessage& message) = 0;
    virtual void OnPlayer(CMessage& message) = 0;
    virtual void OnOther(CMessage& message) = 0;
    virtual void OnGm(CMessage& message) = 0;
    virtual void OnTeam(CMessage& message) = 0;
    virtual void OnOrganizingSystem(CMessage& message) = 0;
    virtual void OnWriteLog(CMessage& message) = 0;
    virtual void OnCountry(CMessage& message) = 0;
    virtual void OnServerAuction(CMessage& message) = 0;
    virtual void OnJjcSystem(CMessage& message) = 0;
    virtual void OnMiscAuction(CMessage& message) = 0;
};

class CMessage final : public CBaseMessage
{
public:
    explicit CMessage(std::int32_t messageType);

    [[nodiscard]] static CreateMessageResult Create(
        std::span<const std::uint8_t> compressed,
        std::uint32_t receiveTimeMs);
    [[nodiscard]] static CreateMessageResult CreateMessageWithoutRLE(
        std::span<const std::uint8_t> wire,
        std::uint32_t receiveTimeMs);

    [[nodiscard]] std::int32_t MessageType() const noexcept;
    [[nodiscard]] CBaseMessage& Base() noexcept;
    [[nodiscard]] std::span<const std::uint8_t> WireBytes() const noexcept;
    void ApplyClientContext(const ServerClientMessageContext& context) noexcept;
    void ApplyMapID(std::int32_t mapId) noexcept;
    [[nodiscard]] std::int32_t MapID() const noexcept;
    [[nodiscard]] std::int32_t SocketID() const noexcept;
    [[nodiscard]] std::uint32_t PeerIPv4() const noexcept;
    [[nodiscard]] std::uint32_t ReceiveTimeMs() const noexcept;

    [[nodiscard]] std::variant<std::vector<std::uint8_t>, SendMessageError>
    ServerEnvelope() const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendToSocket(const ServerCommandHandle* sender, std::int32_t socketId) const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendToMapID(const ServerCommandHandle* sender, std::int32_t mapId) const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendAll(const ServerCommandHandle* sender) const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    Send(ClientSendQueue* sender, bool prioritized = false) const;

    [[nodiscard]] std::int32_t Run(IWorldMessageHandlers& handlers);

private:
    CMessage(std::span<const std::uint8_t, kHeaderSize> header,
             std::span<const std::uint8_t> payload,
             std::uint32_t receiveTimeMs);

    std::int32_t m_MapID{};
    std::int32_t m_SocketID{};
    std::uint32_t m_IP{};
    std::uint32_t m_ReceiveTimeMs{};
};
}
