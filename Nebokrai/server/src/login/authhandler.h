#pragma once

#include "authmanager.h"
#include "loginqueue.h"

#include <cstdint>
#include <optional>
#include <span>
#include <variant>
#include <vector>

/*
 * Owner: loginserver/authhandler.cpp
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * OnResponse RVA 0x000025D0; vtable 0x004847EC. Первый virtual slot точечно
 * проверен в exact EXE: одна инструкция `ret 4`, поэтому OnQuest — no-op.
 *
 * result==0 создаёт TagPwdChecked с пустым world и matrix=false. Ненулевой
 * результат сначала отправляет client 0xAF501 с exact mapping. Только default
 * mapping 7 меняет password-error счётчик; достигнутый ранее limit вызывает
 * синхронный CDKeyBan и удаление счётчика независимо от DB result.
 *
 * Старый handler находил глобальный CGame через GetGame(). Новый non-owning
 * IAuthHandlerContext выражает только три фактических вызова owner-а и не
 * переносит доменную state-machine из CGame/CLoginQueue внутрь handler-а.
 */
namespace Login
{
enum class PasswordFailureOutcomeKind
{
    Disabled,
    Counted,
    BanAttempted,
    BanOwnerMissing,
};

struct PasswordFailureOutcome
{
    PasswordFailureOutcomeKind kind{PasswordFailureOutcomeKind::Disabled};
    std::int32_t failures{};
    bool banSucceeded{};
};

enum class AuthHandlerNoticeKind
{
    ClientResponseFailed,
    CdKeyBanFailed,
    CdKeyBanOwnerMissing,
};

struct AuthHandlerNotice
{
    AuthHandlerNoticeKind kind{};
    std::int32_t socketId{};
    std::vector<std::uint8_t> account;
    std::optional<LoginNet::SendMessageError> sendError;
};

class IAuthHandlerContext
{
public:
    virtual ~IAuthHandlerContext() = default;

    virtual void PushBackPwdChecked(TagPwdChecked checked) = 0;
    [[nodiscard]] virtual std::variant<std::int32_t, LoginNet::SendMessageError>
    SendToClient(const LoginNet::CMessage& message, std::int32_t socketId) = 0;
    [[nodiscard]] virtual PasswordFailureOutcome
    RegisterPasswordFailure(std::span<const std::uint8_t> account) = 0;
    virtual void PushAuthHandlerNotice(AuthHandlerNotice notice) = 0;
};

class AuthHandler final : public IAuthListener
{
public:
    explicit AuthHandler(IAuthHandlerContext& context) noexcept;

    void OnQuest(const AuthQuest& quest) override;
    void OnResponse(const AuthResult& result) override;

private:
    [[nodiscard]] static std::int8_t ResponseCode(std::int32_t result) noexcept;

    IAuthHandlerContext& m_Context;
};
}
