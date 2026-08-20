#pragma once

#include "../basemessage.h"
#include "../serverclient.h"
#include "../servers.h"

#include <cstdint>
#include <memory>
#include <optional>
#include <span>
#include <variant>
#include <vector>

/*
 * Исходный владелец: nets/netbilling/message.cpp / message.h
 *
 * BillingServer EXE/PDB подтверждают отдельный CMessage, 16-байтовое внутреннее
 * сообщение и внешний GameServer-envelope [длина, CRC длины, CRC сообщения,
 * сообщение]. Rust-реконструкция фиксирует create/send/Run и metadata
 * соединения; архивная Linux-версия не является источником поведения.
 */
namespace BillingNet
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

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return message != nullptr;
    }
};

enum class SendMessageError
{
    LengthOutsideLegacyRange,
};

class IBillingMessageHandlers
{
public:
    virtual ~IBillingMessageHandlers() = default;
    virtual void OnBilling(CMessage& message) = 0;
    virtual void OnServer(CMessage& message) = 0;
};

class CMessage final : public CBaseMessage
{
public:
    explicit CMessage(std::int32_t messageType);

    [[nodiscard]] static CreateMessageResult Create(std::span<const std::uint8_t> compressed);
    [[nodiscard]] static CreateMessageResult
    CreateMessageWithoutRLE(std::span<const std::uint8_t> wire);

    [[nodiscard]] std::int32_t MessageType() const noexcept;
    [[nodiscard]] CBaseMessage& Base() noexcept;
    [[nodiscard]] std::span<const std::uint8_t> WireBytes() const noexcept;

    void ApplyClientContext(const ServerClientMessageContext& context);
    [[nodiscard]] std::int32_t SocketID() const noexcept;
    [[nodiscard]] std::int32_t MapID() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> Cdkey() const noexcept;
    [[nodiscard]] std::uint32_t IP() const noexcept;

    [[nodiscard]] std::variant<std::vector<std::uint8_t>, SendMessageError>
    ServerEnvelope() const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendToGS(const ServerCommandHandle& sender, std::int32_t socketId) const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendToAllGS(const ServerCommandHandle& sender) const;

    [[nodiscard]] std::int32_t Run(IBillingMessageHandlers& handlers);

private:
    CMessage(std::span<const std::uint8_t, kHeaderSize> header,
             std::span<const std::uint8_t> payload);

    std::vector<std::uint8_t> m_Cdkey;
    std::int32_t m_MapID{};
    std::int32_t m_SocketID{};
    std::uint32_t m_IP{};
};
}
