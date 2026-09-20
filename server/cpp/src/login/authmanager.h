#pragma once

#include "../nets/netlogin/message.h"
#include "../nets/netlogin/mynetclient_auth.h"

#include <cstddef>
#include <cstdint>
#include <deque>
#include <span>
#include <variant>
#include <vector>

/*
 * Исходный владелец: loginserver/authmanager.cpp / authmanager.h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Подтверждённые RVA: init 0x0001FBD0, send_quest_message 0x0001FBF0,
 * run 0x0001FD20, конструктор AuthQuest 0x0001FF10, removeQuest 0x000201A0,
 * OnResponseAuth 0x00020200, manager ctor/dtor 0x00020350/0x00020390,
 * addQuest 0x000203E0/0x00020460.
 *
 * Pending заявки хранят byte-exact account/password и wrapping boot tick.
 * Duplicate account — no-op. Новая заявка сначала добавляется в хвост, затем
 * отправляется 0xCF501 и только после попытки send вызывается первый listener
 * slot; send-error не откатывает очередь. Вторая исходная форма AddQuest сама
 * ставит текущий legacy tick, а RemoveQuest удаляет первое exact account.
 *
 * Исходный manager держал nullable AuthListener*. Здесь listener передаётся
 * non-owning ссылкой только на время синхронного callback, поэтому порядок и
 * семантика vtable-вызовов сохраняются без отдельного lifetime-state.
 *
 * Timeout использует strict `timeout < now-start`. Просроченная запись не
 * удаляется и timestamp не обновляется: каждый следующий Run до первого
 * обработанного synthetic response снова публикует 0xCF601 в ту же Auth FIFO.
 */
namespace Login
{
class AuthQuest
{
public:
    AuthQuest(std::uint32_t clientIp,
              std::int32_t clientSocketId,
              std::vector<std::uint8_t> account,
              std::vector<std::uint8_t> password,
              std::uint32_t startTimeMs);

    [[nodiscard]] std::uint32_t ClientIP() const noexcept;
    [[nodiscard]] std::int32_t ClientSocketID() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> Account() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> Password() const noexcept;
    [[nodiscard]] std::uint32_t StartTimeMs() const noexcept;

private:
    std::uint32_t m_ClientIP{};
    std::int32_t m_ClientSocketID{};
    std::vector<std::uint8_t> m_Account;
    std::vector<std::uint8_t> m_Password;
    std::uint32_t m_StartTimeMs{};
};

struct AuthResult
{
    std::int32_t result{};
    std::vector<std::uint8_t> account;
    std::uint32_t clientIp{};
    std::int32_t clientSocketId{};
};

class IAuthListener
{
public:
    virtual ~IAuthListener() = default;
    virtual void OnQuest(const AuthQuest& quest) = 0;
    virtual void OnResponse(const AuthResult& result) = 0;
};

enum class AddQuestStatus
{
    Duplicate,
    Added,
};

struct AddQuestResult
{
    AddQuestStatus status{AddQuestStatus::Duplicate};
    std::variant<std::int32_t, LoginNet::SendMessageError> sendResult{std::int32_t{0}};
};

struct AuthRunResult
{
    std::size_t publishedTimeouts{};
};

enum class AuthResponseStatus
{
    Response,
    InvalidResponse,
};

struct AuthResponseResult
{
    AuthResponseStatus status{AuthResponseStatus::InvalidResponse};
    AuthResult response;
};

class AuthManager
{
public:
    AuthManager();

    [[nodiscard]] bool Init(std::uint32_t timeoutMs) noexcept;
    void SetTimeout(std::uint32_t timeoutMs) noexcept;

    [[nodiscard]] AddQuestResult AddQuest(AuthQuest quest,
                                          ClientSendQueue& sender,
                                          IAuthListener& listener);
    [[nodiscard]] AddQuestResult AddQuest(std::uint32_t clientIp,
                                          std::int32_t clientSocketId,
                                          std::span<const std::uint8_t> account,
                                          std::span<const std::uint8_t> password,
                                          ClientSendQueue& sender,
                                          IAuthListener& listener);
    void RemoveQuest(std::span<const std::uint8_t> account);
    [[nodiscard]] AuthRunResult Run(const LoginNet::AuthClientEventPublisher& publisher) const;
    [[nodiscard]] AuthResponseResult OnResponseAuth(LoginNet::CMessage& message,
                                                    IAuthListener& listener);

    [[nodiscard]] std::size_t PendingCount() const noexcept;

    [[nodiscard]] static std::uint32_t LegacyTickMs() noexcept;

private:
    [[nodiscard]] std::variant<std::int32_t, LoginNet::SendMessageError>
    SendQuestMessage(const AuthQuest& quest, ClientSendQueue& sender) const;

    std::deque<AuthQuest> m_Pending;
    std::uint32_t m_TimeoutMs{1000};
};
}
