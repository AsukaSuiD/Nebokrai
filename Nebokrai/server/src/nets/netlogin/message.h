#pragma once

#include "../basemessage.h"
#include "../clients.h"
#include "../serverclient.h"
#include "../servers.h"

#include <cstdint>
#include <memory>
#include <optional>
#include <span>
#include <variant>
#include <vector>

/*
 * Owner: nets/netlogin/message.cpp / message.h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Исходный путь PDB:
 * d:\complite_version\fengyun_russia\trunk\nets\netlogin\message.cpp
 * RVA: Run 0x00065490, ctor 0x00065540, CreateMessage 0x000655C0,
 * CreateMessageWithoutRLE 0x00065710, GetString 0x000657D0,
 * client send 0x00065190/0x000651E0, World send
 * 0x00065230/0x000652C0/0x00065350, Auth send 0x000653E0.
 *
 * Client wire: [rle_len + 4, rle(full CBaseMessage)]. World/Auth wire:
 * [total_len, crc(total_len), crc(message), message]. Только SendToAS имел
 * отдельную сериализацию общего scratch-buffer; в новом C++ она сохраняется
 * mutex вокруг build/queue, но сам scratch заменён owned std::vector.
 *
 * RLE create сохраняет старый threshold 0x20001 и output capacity 0x100000
 * либо compressed_len*8. Malformed trailing marker и короткий decoded header
 * остаются типизированными неизвестными границами, а не получают unsafe.
 *
 * Login-specific типы находятся в namespace LoginNet, потому что в исходных
 * отдельных EXE каждый сервис имел собственный глобальный CMessage, а единый
 * современный процесс не может корректно иметь несколько одноимённых типов.
 */
namespace LoginNet
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
    RleEmptyInputReactionUnknown,
};

class ILoginMessageHandlers
{
public:
    virtual ~ILoginMessageHandlers() = default;
    virtual void OnAuth(CMessage& message) = 0;
    virtual void OnGM(CMessage& message) = 0;
    virtual void OnGMA(CMessage& message) = 0;
    virtual void OnLog(CMessage& message) = 0;
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
    void SetMessageType(std::int32_t messageType) noexcept;
    [[nodiscard]] CBaseMessage& Base() noexcept;
    [[nodiscard]] const CBaseMessage& Base() const noexcept;
    [[nodiscard]] std::vector<std::uint8_t> GetString();
    [[nodiscard]] std::span<const std::uint8_t> WireBytes() const noexcept;

    void ApplyClientContext(const ServerClientMessageContext& context);
    void ApplyMapID(std::int32_t mapId) noexcept;

    [[nodiscard]] std::int32_t SocketID() const noexcept;
    [[nodiscard]] std::int32_t MapID() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> Cdkey() const noexcept;
    [[nodiscard]] std::uint32_t IP() const noexcept;

    [[nodiscard]] std::variant<std::vector<std::uint8_t>, SendMessageError>
    ClientEnvelope() const;
    [[nodiscard]] std::variant<std::vector<std::uint8_t>, SendMessageError>
    ServerEnvelope() const;

    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendToClientSocket(const ServerCommandHandle& sender, std::int32_t socketId) const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendToClientCdkey(const ServerCommandHandle& sender,
                      std::span<const std::uint8_t> cdkey) const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendToWorldSocket(const ServerCommandHandle& sender, std::int32_t socketId) const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendToWorldMap(const ServerCommandHandle& sender, std::int32_t mapId) const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendAllWorld(const ServerCommandHandle& sender) const;
    [[nodiscard]] std::variant<std::int32_t, SendMessageError>
    SendToAuth(ClientSendQueue& sender) const;

    [[nodiscard]] std::int32_t Run(ILoginMessageHandlers& handlers);

private:
    CMessage(std::span<const std::uint8_t, kHeaderSize> header,
             std::span<const std::uint8_t> payload);

    std::vector<std::uint8_t> m_Cdkey;
    std::int32_t m_SocketID{};
    std::int32_t m_MapID{};
    std::uint32_t m_IP{};
};
}
