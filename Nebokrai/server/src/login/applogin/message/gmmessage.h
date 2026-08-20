#pragma once

#include "../../../nets/netlogin/message.h"

#include <cstdint>
#include <optional>
#include <span>
#include <variant>

/*
 * Исходный владелец: loginserver/applogin/message/gmmessage.cpp
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * RVA OnGMMessage: 0x0007F0A0.
 * Путь владельца в PDB: d:\complite_version\fengyun_russia\trunk\server\loginserver\applogin\message\gmmessage.cpp
 *
 * Единственный подтверждённый opcode 0x20001 читает account через GetStr с
 * limit 0x100, затем signed Windows long duration и синхронно вызывает
 * CRsCDKey::CDKeyBan. Возвращённый bool оригинальный handler игнорировал;
 * здесь он только сохраняется в typed outcome без нового side effect.
 * Неизвестный opcode остаётся no-op.
 *
 * Глобальный GetGame заменён узким IGmMessageContext. Добавленные в старом
 * Linux-доноре коды списка блокировок 0x20002/0x20003 отсутствуют у точного владельца и
 * намеренно не переносятся. Stack char[256], STL cleanup и compiler noise
 * заменены owned bytes и стандартными контейнерами.
 */
namespace Login
{
class IGmMessageContext
{
public:
    virtual ~IGmMessageContext() = default;

    // nullopt означает, что CGame ещё не присоединил исходного владельца CRsCDKey.
    [[nodiscard]] virtual std::optional<bool>
    CdKeyBan(std::span<const std::uint8_t> account,
             std::int32_t durationMinutes) = 0;
};

enum class GmMessageOutcomeKind
{
    BanAttempted,
    Unsupported,
};

struct GmMessageOutcome
{
    GmMessageOutcomeKind kind{GmMessageOutcomeKind::Unsupported};
    std::int32_t messageType{};
    bool banSucceeded{};
};

enum class GmMessageError
{
    DatabaseOwnerMissing,
};

using GmMessageResult = std::variant<GmMessageOutcome, GmMessageError>;

class GmMessageHandler
{
public:
    explicit GmMessageHandler(IGmMessageContext& context) noexcept;

    [[nodiscard]] GmMessageResult OnGMMessage(LoginNet::CMessage& message);

private:
    IGmMessageContext& m_Context;
};
}
