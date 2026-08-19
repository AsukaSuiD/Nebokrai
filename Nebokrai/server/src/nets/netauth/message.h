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
 * Owner: nets/netauth/message.cpp
 *
 * AuthServer EXE/PDB: authserver.exe/authserver.pdb. Существенные RVA:
 * ctor 0x000136B0, Run 0x00013710, CreateMessageWithoutRLE 0x00013760,
 * SendToLogin 0x00013830, GetStr 0x000138C0, InitMsgFuncPool 0x000141A0.
 *
 * Внутреннее Auth-сообщение — общий 16-байтовый CBaseMessage. Внешний server
 * envelope строго: [total_len, crc(total_len), crc(message), message]. Create
 * сохраняет четыре слова входного header, затем нормализует слово длины по
 * реально добавленному payload. Runtime metadata socket/map/CD-key/IP/tick не
 * являются частью wire.
 *
 * Неизвестный opcode у Run является no-op. Единственная не закрытая запись
 * исходной handler map — 0x10F101: экспорт связывает её с operator_delete, что
 * при обычном последующем уничтожении сообщения выглядело бы как double-free.
 * До точечной проверки target/dreachability временный handler не создаётся.
 */

enum class AuthCreateMessageError
{
    EmptyInputReactionUnknown,
    HeaderTooShortReactionUnknown,
    InputOutsideLegacyRange,
};

enum class AuthSendMessageError
{
    LengthOutsideLegacyRange,
};

enum class AuthMessageKind
{
    LoginServerConnected,
    LoginServerDisconnected,
    AuthenticateAccount,
    AuthenticateAccountExtended,
    GetLoginServerInfo,
    UpdateServerInfoResponse,
    GmKickPlayer,
    KickPlayerResponse,
    GmLockAccount,
};

enum class AuthDispatchStatus
{
    Ok,
    DeleteHandlerAt10F101Unresolved,
    OutgoingMessageTooLarge,
};

class CMessage;

class IAuthMessageHandler
{
public:
    virtual ~IAuthMessageHandler() = default;

    [[nodiscard]] virtual AuthDispatchStatus Handle(AuthMessageKind kind,
                                                    CMessage& message,
                                                    const ServerCommandHandle& sender) = 0;
};

struct AuthCreateMessageResult
{
    std::unique_ptr<CMessage> message;
    std::optional<AuthCreateMessageError> error;

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return message != nullptr;
    }
};

struct AuthEnvelopeResult
{
    std::vector<std::uint8_t> bytes;
    std::optional<AuthSendMessageError> error;

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return !error.has_value();
    }
};

class CMessage final : public CBaseMessage
{
public:
    explicit CMessage(std::int32_t messageType);

    [[nodiscard]] static AuthCreateMessageResult
    CreateMessageWithoutRLE(std::span<const std::uint8_t> wire, std::uint32_t recvTimeMs);

    [[nodiscard]] std::int32_t MessageType() const noexcept;
    [[nodiscard]] CBaseMessage& Base() noexcept;
    [[nodiscard]] const CBaseMessage& Base() const noexcept;
    [[nodiscard]] std::vector<std::uint8_t> GetStr();
    [[nodiscard]] std::span<const std::uint8_t> WireBytes() const noexcept;

    void ApplyClientContext(const ServerClientMessageContext& context);
    void ApplySocketContext(std::int32_t socketId, std::uint32_t peerIPv4) noexcept;

    [[nodiscard]] std::int32_t SocketID() const noexcept;
    [[nodiscard]] std::int32_t MapID() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> Cdkey() const noexcept;
    [[nodiscard]] std::uint32_t IP() const noexcept;
    [[nodiscard]] std::uint32_t RecvTimeMs() const noexcept;

    [[nodiscard]] AuthEnvelopeResult ServerEnvelope() const;
    [[nodiscard]] std::variant<std::int32_t, AuthSendMessageError>
    SendToLogin(const ServerCommandHandle& sender, std::int32_t socketId) const;

    [[nodiscard]] AuthDispatchStatus Run(IAuthMessageHandler& handler,
                                         const ServerCommandHandle& sender);

private:
    CMessage(std::span<const std::uint8_t, kHeaderSize> header,
             std::span<const std::uint8_t> payload,
             std::uint32_t recvTimeMs);

    std::vector<std::uint8_t> m_Cdkey;
    std::int32_t m_SocketID{};
    std::int32_t m_MapID{};
    std::uint32_t m_RecvTimeMs{};
    std::uint32_t m_IP{};
};
