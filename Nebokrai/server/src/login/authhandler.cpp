#include "authhandler.h"

#include <utility>

namespace Login
{
namespace
{
constexpr std::int32_t kAuthFailedMessageType = 0x000AF501;
}

AuthHandler::AuthHandler(IAuthHandlerContext& context) noexcept
    : m_Context(context)
{
}

void AuthHandler::OnQuest(const AuthQuest& quest)
{
    static_cast<void>(quest);
    // Точный виртуальный слот 0x004664B0: ret 4.
}

void AuthHandler::OnResponse(const AuthResult& result)
{
    if (result.result == 0) {
        m_Context.PushBackPwdChecked(TagPwdChecked(
            result.clientSocketId,
            result.clientIp,
            result.account,
            {},
            false));
        return;
    }

    const std::int8_t responseCode = ResponseCode(result.result);
    LoginNet::CMessage response(kAuthFailedMessageType);
    response.Base().Add(static_cast<char>(responseCode));
    auto sendResult = m_Context.SendToClient(response, result.clientSocketId);
    if (const auto* error = std::get_if<LoginNet::SendMessageError>(&sendResult)) {
        m_Context.PushAuthHandlerNotice(AuthHandlerNotice{
            .kind = AuthHandlerNoticeKind::ClientResponseFailed,
            .socketId = result.clientSocketId,
            .account = {},
            .sendError = *error,
        });
    }

    if (responseCode != 7) {
        return;
    }

    const PasswordFailureOutcome failure =
        m_Context.RegisterPasswordFailure(result.account);
    if (failure.kind == PasswordFailureOutcomeKind::BanAttempted &&
        !failure.banSucceeded) {
        m_Context.PushAuthHandlerNotice(AuthHandlerNotice{
            .kind = AuthHandlerNoticeKind::CdKeyBanFailed,
            .account = result.account,
        });
    } else if (failure.kind == PasswordFailureOutcomeKind::BanOwnerMissing) {
        m_Context.PushAuthHandlerNotice(AuthHandlerNotice{
            .kind = AuthHandlerNoticeKind::CdKeyBanOwnerMissing,
            .account = result.account,
        });
    }
}

std::int8_t AuthHandler::ResponseCode(std::int32_t result) noexcept
{
    switch (result) {
    case 2: return 5;
    case 3: return 6;
    case 4: return 73; // 'I'
    case 5: return 18;
    case 6: return 63; // '?'
    case 8: return 82; // 'R'
    case 9: return 83; // 'S'
    case 10: return 84; // 'T'
    case 11: return 87; // 'W'
    case 12: return 88; // 'X'
    default: return 7;
    }
}
}
