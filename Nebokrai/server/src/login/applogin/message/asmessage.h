#pragma once

#include "../../authmanager.h"
#include "../../../nets/netlogin/message.h"

#include <cstdint>
#include <optional>
#include <span>
#include <variant>
#include <vector>

/*
 * Owner: loginserver/applogin/message/asmessage.cpp
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Исходный путь PDB:
 * d:\complite_version\fengyun_russia\trunk\server\loginserver\applogin\message\asmessage.cpp
 * RVA: OnGMAKickPlayer 0x0007F110, OnGMAMessage 0x0007F2A0,
 * OnASMessage 0x0007F320.
 *
 * OnASMessage сохраняет три существенных пути: 0xCF301 сначала закрывает
 * текущий Auth client и только затем запускает reconnect-owner, игнорируя
 * исходный bool результата StartReconnectThread; 0xCF601 передаётся
 * AuthManager; диапазон GMA уходит в OnGMAMessage. Исторический
 * 0xCF302 содержал process-local pointer и потому не является допустимым wire
 * сообщением в безопасной реконструкции: современный reconnect передаёт typed
 * event через Auth FIFO отдельно от CMessage.
 *
 * GMA сохраняет исходное изменение opcode перед пересылкой, byte-exact имена
 * world/account и sentinel world id -1. Оба GetStr имеют limit 0x100. Для
 * invalid world diagnostic vararg с именем мира подтверждён машинным кодом
 * LoginServer 0x0047F1CF..0x0047F1E4.
 *
 * Старые глобальные g_pGame/gAuthMgr не воспроизводятся. AuthManager остаётся
 * явным owner, а IAsMessageContext выражает только фактические CGame-вызовы
 * этого файла. Это техническая граница владения, не новая доменная архитектура.
 */
namespace Login
{
enum class AsRouteErrorKind
{
    MissingAuthServer,
    MissingWorldServer,
    SendMessage,
};

struct AsRouteError
{
    AsRouteErrorKind kind{};
    std::optional<LoginNet::SendMessageError> sendError;
};

using AsRouteResult = std::variant<std::int32_t, AsRouteError>;

class IAsMessageContext
{
public:
    virtual ~IAsMessageContext() = default;

    virtual void DisconnectAuth() = 0;
    [[nodiscard]] virtual bool StartReconnectThread() = 0;

    [[nodiscard]] virtual std::int32_t
    WorldIDByName(std::span<const std::uint8_t> worldName) const = 0;
    [[nodiscard]] virtual std::int32_t AreaID() const noexcept = 0;

    [[nodiscard]] virtual AsRouteResult SendToAuth(const LoginNet::CMessage& message) = 0;
    [[nodiscard]] virtual AsRouteResult
    SendToWorld(const LoginNet::CMessage& message, std::int32_t worldId) = 0;
    [[nodiscard]] virtual AsRouteResult SendAllWorld(const LoginNet::CMessage& message) = 0;
};

enum class UnknownAsMessageOwner
{
    Auth,
    Gma,
};

struct AsMessageHandled
{
};

struct AsMessageUnknown
{
    UnknownAsMessageOwner owner{};
    std::int32_t messageType{};
};

using AsMessageOutcome = std::variant<AsMessageHandled,
                                      AuthResponseResult,
                                      AsMessageUnknown>;

enum class AsMessageErrorKind
{
    Route,
    LegacyReconnectPointerOnWire,
};

struct AsMessageError
{
    AsMessageErrorKind kind{};
    std::optional<AsRouteError> routeError;
};

using AsMessageResult = std::variant<AsMessageOutcome, AsMessageError>;

class AsMessageHandler
{
public:
    AsMessageHandler(IAsMessageContext& context,
                     AuthManager& authManager,
                     IAuthListener& authListener) noexcept;

    [[nodiscard]] AsMessageResult OnASMessage(LoginNet::CMessage& message);
    [[nodiscard]] AsMessageResult OnGMAMessage(LoginNet::CMessage& message);

private:
    [[nodiscard]] AsMessageResult OnGMAKickPlayer(LoginNet::CMessage& message);
    [[nodiscard]] static AsMessageResult RouteResult(AsRouteResult result);

    IAsMessageContext& m_Context;
    AuthManager& m_AuthManager;
    IAuthListener& m_AuthListener;
};
}
