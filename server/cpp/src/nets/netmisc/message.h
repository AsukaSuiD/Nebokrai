#pragma once

#include "../basemessage.h"
#include "../clients.h"

#include <cstdint>
#include <memory>
#include <optional>
#include <span>
#include <variant>
#include <vector>

/*
 * Исходный владелец: nets/netmisc/message.cpp / message.h.
 *
 * MiscServer EXE/PDB и Rust-реконструкция подтверждают 16-байтовое внутреннее
 * сообщение, внешний envelope [длина, CRC длины, CRC сообщения, сообщение] и
 * маршрутизацию семейств 0x14ED00/0x16EA00. Старые общие scratch-буферы
 * заменены владеющим std::vector; wire-формат и порядок CRC сохранены.
 */
namespace MiscNet
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

class IMiscMessageHandlers
{
public:
    virtual ~IMiscMessageHandlers() = default;
    virtual void OnWorldAuction(CMessage& message) = 0;
    virtual void OnMiscFunction(CMessage& message) = 0;
    virtual void OnOther(CMessage& message) = 0;
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
    [[nodiscard]] std::uint32_t ReceiveTimeMs() const noexcept;

    [[nodiscard]] std::variant<std::vector<std::uint8_t>, SendMessageError>
    ServerEnvelope() const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    Send(ClientSendQueue* sender, bool prioritized) const;

    [[nodiscard]] std::int32_t Run(IMiscMessageHandlers& handlers);

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
