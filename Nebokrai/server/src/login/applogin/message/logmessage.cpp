#include "logmessage.h"

#include <algorithm>
#include <array>
#include <filesystem>
#include <optional>
#include <utility>
#include <vector>

namespace Login
{
namespace
{
constexpr std::int32_t kSyntheticDisconnectMessageType = 0x00010001;
constexpr std::int32_t kWorldLoginResultMessageType = 0x0001FF01;
constexpr std::int32_t kWorldPlayerBaseResponseMessageType = 0x0001FF02;
constexpr std::int32_t kWorldDeleteRoleResponseMessageType = 0x0001FF03;
constexpr std::int32_t kWorldRestoreRoleResponseMessageType = 0x0001FF04;
constexpr std::int32_t kWorldCreateRoleResponseMessageType = 0x0001FF05;
constexpr std::int32_t kWorldLeaveResultMessageType = 0x0001FF06;
constexpr std::int32_t kWorldResponse1FF07MessageType = 0x0001FF07;
constexpr std::int32_t kClientLoginRequestMessageType = 0x0002FD01;
constexpr std::int32_t kPlayerListRequestMessageType = 0x0002FD02;
constexpr std::int32_t kPlayerDataRequestMessageType = 0x0002FD03;
constexpr std::int32_t kCreateRoleRequestMessageType = 0x0002FD04;
constexpr std::int32_t kDeleteRoleRequestMessageType = 0x0002FD05;
constexpr std::int32_t kRestoreRoleRequestMessageType = 0x0002FD06;
constexpr std::int32_t kClientCleanupMessageType = 0x0002FD07;
constexpr std::int32_t kMatrixAnswerMessageType = 0x0002FD08;
constexpr std::int32_t kValidCodeAnswerMessageType = 0x0002FD09;
constexpr std::int32_t kValidCodeRefreshMessageType = 0x0002FD0A;
constexpr std::int32_t kExtendedLoginRequestMessageType = 0x0002FD0B;
constexpr std::int32_t kWorldListRequestMessageType = 0x0002FD0C;

constexpr std::int32_t kWorldClientLostMessageType = 0x0004FB06;
constexpr std::int32_t kLoginResponseMessageType = 0x000AF501;
constexpr std::int32_t kPlayerBaseResponseMessageType = 0x000AF502;
constexpr std::int32_t kLoginResultResponseMessageType = 0x000AF503;
constexpr std::int32_t kCreateRoleResponseMessageType = 0x000AF504;
constexpr std::int32_t kDeleteRoleResponseMessageType = 0x000AF505;
constexpr std::int32_t kRestoreRoleResponseMessageType = 0x000AF506;
constexpr std::int32_t kResponseAF508MessageType = 0x000AF508;
constexpr std::int32_t kExtendedLoginResponseMessageType = 0x000AF50A;
constexpr std::int32_t kWorldListResponseMessageType = 0x000AF50B;

constexpr std::size_t kAccountLimit = 0x14U;
constexpr std::size_t kExtendedAccountLimit = 0x20U;
constexpr std::size_t kRoleFieldLimit = 0x100U;
constexpr std::size_t kWorldServerLimit = 0x14U;
constexpr std::size_t kValidCodeLimit = 10U;
constexpr std::size_t kPasswordDigestLength = 0x10U;
constexpr std::size_t kLoginUnusedFieldLimit = 0x40U;
constexpr std::size_t kExtendedLoginWorldLimit = 0x20U;
constexpr std::size_t kExtendedLoginPasswordLimit = 0x104U;
constexpr std::uint8_t kWorldLoginSuccessStatus = 0x1DU;
constexpr std::int32_t kClientLoginMarker = 6;

struct PasswordDigestRead
{
    std::optional<std::vector<std::uint8_t>> digest;
    std::optional<LogMessageError> error;
};

std::vector<std::uint8_t> GetBoundedString(LoginNet::CMessage& message,
                                           std::size_t maximum)
{
    return message.Base().GetStrBytes(maximum).value_or(std::vector<std::uint8_t>{});
}

std::span<const std::uint8_t> LegacyCStringPrefix(std::span<const std::uint8_t> value)
{
    const auto terminator = std::find(value.begin(), value.end(), std::uint8_t{0});
    return value.first(static_cast<std::size_t>(std::distance(value.begin(), terminator)));
}

std::vector<std::uint8_t> MetadataAccount(const LoginNet::CMessage& message)
{
    const auto prefix = LegacyCStringPrefix(message.Cdkey());
    return {prefix.begin(), prefix.end()};
}

void AddLegacyString(LoginNet::CMessage& message, std::span<const std::uint8_t> value)
{
    const auto prefix = LegacyCStringPrefix(value);
    if (!prefix.empty()) {
        message.Base().Add(prefix.data(), static_cast<std::int32_t>(prefix.size()));
    }
    message.Base().Add(std::uint8_t{0});
}

void RemoveSpaces(std::vector<std::uint8_t>& value)
{
    value.erase(std::remove(value.begin(), value.end(), static_cast<std::uint8_t>(' ')),
                value.end());
}

void LowerAscii(std::vector<std::uint8_t>& value)
{
    for (auto& byte : value) {
        // BLOCKED_MISSING_FACT: exact CRT tolower для high-bit signed-char
        // зависел от активной ANSI locale. ASCII-часть доказана однозначно.
        if (byte >= 'A' && byte <= 'Z') {
            byte = static_cast<std::uint8_t>(byte - 'A' + 'a');
        }
    }
}

bool HasForbiddenAccountByte(std::span<const std::uint8_t> account)
{
    return std::any_of(account.begin(), account.end(), [](std::uint8_t byte) {
        return byte == '\'' || byte == '=' || byte == ' ';
    });
}

std::optional<std::span<const std::uint8_t>> OptionalWorldSpan(
    const std::optional<std::vector<std::uint8_t>>& world)
{
    if (!world) {
        return std::nullopt;
    }
    return std::span<const std::uint8_t>(world->data(), world->size());
}

void SendCode(ILogMessageContext& context, std::int32_t socketId, std::uint8_t code)
{
    LoginNet::CMessage response(kLoginResponseMessageType);
    response.Base().Add(static_cast<char>(code));
    context.SendToClient(response, socketId);
}

void SendExtendedLoginRejection(ILogMessageContext& context, std::int32_t socketId)
{
    LoginNet::CMessage response(kExtendedLoginResponseMessageType);
    response.Base().Add(kClientLoginMarker);
    AddLegacyString(response, std::span<const std::uint8_t>{});
    AddLegacyString(response, std::span<const std::uint8_t>{});
    context.SendToClient(response, socketId);
}

PasswordDigestRead ReadPasswordDigest(LoginNet::CMessage& message)
{
    const std::int32_t readPtr = message.Base().GetReadPtr();
    const std::size_t dataSize = message.Base().DataSize();
    const std::size_t cursor = readPtr < 0 ? dataSize : static_cast<std::size_t>(readPtr);
    const std::size_t remaining = cursor <= dataSize ? dataSize - cursor : 0U;
    if (remaining < kPasswordDigestLength) {
        return {};
    }

    const std::int32_t declaredLength = message.Base().GetLong();
    if (declaredLength != static_cast<std::int32_t>(kPasswordDigestLength)) {
        return {};
    }

    const std::size_t available = remaining - sizeof(std::int32_t);
    if (available < kPasswordDigestLength) {
        return PasswordDigestRead{
            .error = LogMessageError{
                .kind = LogMessageErrorKind::PasswordDigestTruncatedReactionUnknown,
                .actualLength = available,
                .requiredLength = kPasswordDigestLength,
            },
        };
    }

    std::vector<std::uint8_t> digest(kPasswordDigestLength);
    if (message.Base().Get(digest.data(), static_cast<std::int32_t>(digest.size())) == nullptr) {
        return {};
    }
    return PasswordDigestRead{.digest = std::move(digest)};
}
}

LogMessageHandler::LogMessageHandler(ILogMessageContext& context,
                                     CLoginQueue& queue) noexcept
    : m_Context(context), m_Queue(queue)
{
}

std::optional<LogMessageError> LogMessageHandler::OnLogMessage(LoginNet::CMessage& message)
{
    switch (message.MessageType()) {
    case kSyntheticDisconnectMessageType:
        OnSyntheticDisconnect(message);
        return std::nullopt;
    case kWorldLoginResultMessageType:
        OnWorldLoginResult(message);
        return std::nullopt;
    case kWorldPlayerBaseResponseMessageType:
        OnWorldPlayerBaseResponse(message);
        return std::nullopt;
    case kWorldDeleteRoleResponseMessageType:
        OnWorldDeleteRoleResponse(message);
        return std::nullopt;
    case kWorldRestoreRoleResponseMessageType:
        OnWorldRestoreRoleResponse(message);
        return std::nullopt;
    case kWorldCreateRoleResponseMessageType:
        OnWorldCreateRoleResponse(message);
        return std::nullopt;
    case kWorldLeaveResultMessageType:
        OnWorldLeaveResult(message);
        return std::nullopt;
    case kWorldResponse1FF07MessageType:
        OnWorldResponse1FF07(message);
        return std::nullopt;
    case kClientLoginRequestMessageType:
        return OnClientLoginRequest(message);
    case kPlayerListRequestMessageType:
        OnPlayerListRequest(message);
        return std::nullopt;
    case kPlayerDataRequestMessageType:
        OnPlayerDataRequest(message);
        return std::nullopt;
    case kCreateRoleRequestMessageType:
        OnCreateRoleRequest(message);
        return std::nullopt;
    case kDeleteRoleRequestMessageType:
        OnDeleteRoleRequest(message);
        return std::nullopt;
    case kRestoreRoleRequestMessageType:
        OnRestoreRoleRequest(message);
        return std::nullopt;
    case kClientCleanupMessageType:
        OnClientCleanup(message);
        return std::nullopt;
    case kMatrixAnswerMessageType:
        return OnMatrixAnswer(message);
    case kValidCodeAnswerMessageType:
        return OnValidCodeAnswer(message);
    case kValidCodeRefreshMessageType:
        return OnValidCodeRefresh(message);
    case kExtendedLoginRequestMessageType:
        return OnExtendedLoginRequest(message);
    case kWorldListRequestMessageType:
        OnWorldListRequest(message);
        return std::nullopt;
    default:
        return std::nullopt;
    }
}

std::optional<LogMessageError> LogMessageHandler::OnClientLoginRequest(
    LoginNet::CMessage& message)
{
    const std::int32_t marker = message.Base().GetLong();
    const std::int32_t clientVersion = message.Base().GetLong();
    auto account = GetBoundedString(message, kExtendedAccountLimit);
    RemoveSpaces(account);
    if (account.empty() || account.size() >= kExtendedAccountLimit) {
        return std::nullopt;
    }

    auto passwordRead = ReadPasswordDigest(message);
    if (passwordRead.error) {
        return passwordRead.error;
    }
    if (!passwordRead.digest) {
        return std::nullopt;
    }

    if (marker != kClientLoginMarker) {
        SendCode(m_Context, message.SocketID(), 4);
        return std::nullopt;
    }
    const auto serverVersion = m_Context.ServerVersion();
    if (!serverVersion) {
        return LogMessageError{.kind = LogMessageErrorKind::LoginServerVersionMissing};
    }
    if (clientVersion != *serverVersion) {
        SendCode(m_Context, message.SocketID(), 4);
        return std::nullopt;
    }
    if (HasForbiddenAccountByte(account)) {
        SendCode(m_Context, message.SocketID(), 5);
        return std::nullopt;
    }

    const std::int16_t clientCode = message.Base().GetShort();
    const std::int32_t encryptionKey = message.Base().GetLong();
    static_cast<void>(GetBoundedString(message, kLoginUnusedFieldLimit));
    const auto worldServer = GetBoundedString(message, kWorldServerLimit);
    LowerAscii(account);

    m_Queue.AddQuestCdkey(message.SocketID(),
                          message.IP(),
                          0,
                          clientVersion,
                          account,
                          *passwordRead.digest,
                          clientCode,
                          encryptionKey,
                          worldServer);
    return std::nullopt;
}

std::optional<LogMessageError> LogMessageHandler::OnExtendedLoginRequest(
    LoginNet::CMessage& message)
{
    const std::int32_t marker = message.Base().GetLong();
    const std::int32_t clientVersion = message.Base().GetLong();
    const auto worldServer = GetBoundedString(message, kExtendedLoginWorldLimit);
    auto account = GetBoundedString(message, kExtendedAccountLimit);
    const auto passwordSource = GetBoundedString(message, kExtendedLoginPasswordLimit);
    if (passwordSource.empty()) {
        return std::nullopt;
    }
    if (account.empty()) {
        return std::nullopt;
    }
    RemoveSpaces(account);

    const std::size_t compactedLength = static_cast<std::size_t>(std::count_if(
        passwordSource.begin(), passwordSource.end(),
        [](std::uint8_t byte) { return byte != static_cast<std::uint8_t>(' '); }));
    std::vector<std::uint8_t> passwordDigest(
        passwordSource.begin(),
        passwordSource.begin() + static_cast<std::ptrdiff_t>(compactedLength));
    if (compactedLength == passwordSource.size()) {
        passwordDigest.push_back(0);
    }

    if (marker != kClientLoginMarker) {
        SendExtendedLoginRejection(m_Context, message.SocketID());
        return std::nullopt;
    }
    const auto serverVersion = m_Context.ServerVersion();
    if (!serverVersion) {
        return LogMessageError{.kind = LogMessageErrorKind::LoginServerVersionMissing};
    }
    if (clientVersion != *serverVersion) {
        SendExtendedLoginRejection(m_Context, message.SocketID());
        return std::nullopt;
    }

    m_Queue.AddQuestCdkey(message.SocketID(),
                          message.IP(),
                          1,
                          clientVersion,
                          account,
                          passwordDigest,
                          0,
                          0,
                          worldServer);
    return std::nullopt;
}

void LogMessageHandler::OnPlayerListRequest(LoginNet::CMessage& message)
{
    const auto worldServer = GetBoundedString(message, kWorldServerLimit);
    const auto account = MetadataAccount(message);
    static_cast<void>(m_Queue.AddQuestPlayerList(
        m_Context, message.SocketID(), account, worldServer));
}

void LogMessageHandler::OnPlayerDataRequest(LoginNet::CMessage& message)
{
    const std::int32_t playerId = message.Base().GetLong();
    const auto account = MetadataAccount(message);
    static_cast<void>(m_Queue.AddQuestPlayerData(
        m_Context, message.SocketID(), account, playerId, message.IP()));
}

void LogMessageHandler::OnCreateRoleRequest(LoginNet::CMessage& message)
{
    const auto account = MetadataAccount(message);
    const auto worldServer = m_Context.LoginCdkeyWorldServer(account);
    m_Context.L2WCreateRoleSend(OptionalWorldSpan(worldServer), account, message);
}

void LogMessageHandler::OnDeleteRoleRequest(LoginNet::CMessage& message)
{
    const auto account = MetadataAccount(message);
    const auto worldServer = m_Context.LoginCdkeyWorldServer(account);
    const std::int32_t playerId = message.Base().GetLong();
    m_Context.L2WDeleteRoleSend(
        OptionalWorldSpan(worldServer), account, playerId, message.IP());
}

void LogMessageHandler::OnRestoreRoleRequest(LoginNet::CMessage& message)
{
    const auto account = MetadataAccount(message);
    const auto worldServer = m_Context.LoginCdkeyWorldServer(account);
    const std::uint32_t playerId = static_cast<std::uint32_t>(message.Base().GetLong());
    m_Context.L2WRestoreRoleSend(OptionalWorldSpan(worldServer), account, playerId);
}

void LogMessageHandler::OnClientCleanup(LoginNet::CMessage& message)
{
    const std::int32_t status = message.Base().GetLong();
    if (status != 0) {
        return;
    }

    const auto account = MetadataAccount(message);
    const auto worldServer = m_Context.LoginCdkeyWorldServer(account);
    const std::int32_t worldId = worldServer ? m_Context.WorldIDByName(*worldServer) : -1;
    if (worldId != -1) {
        LoginNet::CMessage notice(kWorldClientLostMessageType);
        AddLegacyString(notice, account);
        m_Context.SendToWorld(notice, worldId);
    }

    m_Context.ClearCdkey(account);
    static_cast<void>(m_Queue.OnClientLost(account));
    m_Context.AccountLeaveLog(account);
}

std::optional<LogMessageError> LogMessageHandler::OnMatrixAnswer(
    LoginNet::CMessage& message)
{
    auto account = GetBoundedString(message, kAccountLimit);
    if (account.empty()) {
        return std::nullopt;
    }
    LowerAscii(account);

    const std::int32_t readPtr = message.Base().GetReadPtr();
    const std::size_t dataSize = message.Base().DataSize();
    const std::size_t cursor = readPtr < 0 ? dataSize : static_cast<std::size_t>(readPtr);
    const std::size_t remaining = cursor <= dataSize ? dataSize - cursor : 0U;
    std::array<std::uint8_t, 3> answer{};
    if (message.Base().Get(answer.data(), static_cast<std::int32_t>(answer.size())) == nullptr) {
        return LogMessageError{
            .kind = LogMessageErrorKind::MatrixAnswerTooShortReactionUnknown,
            .actualLength = remaining,
            .requiredLength = answer.size(),
        };
    }

    const auto validation = m_Queue.ValidateMatrix(
        m_Context,
        message.SocketID(),
        message.IP(),
        account,
        std::span<const std::uint8_t, 3>(answer));
    switch (validation.kind) {
    case MatrixValidationOutcomeKind::Accepted: {
        m_Queue.DelValidCode(account);
        if (!m_Context.LoginCdkeyWorldServer(account).has_value()) {
            return std::nullopt;
        }
        TagPwdChecked checked(message.SocketID(), message.IP(), account, {}, false);
        m_Context.EnterGame(checked, m_Queue.IsInNoQueueList(account));
        return std::nullopt;
    }
    case MatrixValidationOutcomeKind::Rejected:
        m_Queue.AddValidErr(account, m_Context.ValidErrorStayTimeMs());
        SendCode(m_Context, message.SocketID(), static_cast<std::uint8_t>('D'));
        return std::nullopt;
    case MatrixValidationOutcomeKind::OwnerMissing:
        return LogMessageError{.kind = LogMessageErrorKind::MatrixDatabaseOwnerMissing};
    case MatrixValidationOutcomeKind::ValueTooShort:
        return LogMessageError{
            .kind = LogMessageErrorKind::MatrixDatabaseValueTooShort,
            .actualLength = validation.actualLength,
            .requiredLength = validation.requiredLength,
        };
    }
    return std::nullopt;
}

std::optional<LogMessageError> LogMessageHandler::OnValidCodeAnswer(
    LoginNet::CMessage& message)
{
    auto account = GetBoundedString(message, kAccountLimit);
    if (m_Queue.CheckMsgInfo(account, message.SocketID()) == CheckMessageInfo::SocketMismatch) {
        return std::nullopt;
    }
    if (account.empty()) {
        return std::nullopt;
    }
    LowerAscii(account);
    const auto suppliedCode = GetBoundedString(message, kValidCodeLimit);

    auto validation = m_Queue.ValidateValidCode(
        message.SocketID(), message.IP(), suppliedCode, account);
    if (!validation.accepted) {
        m_Queue.AddValidErr(account, m_Context.ValidErrorStayTimeMs());
        SendCode(m_Context, message.SocketID(), static_cast<std::uint8_t>('L'));
        return std::nullopt;
    }

    m_Queue.DelValidCode(account);
    TagPwdChecked checked(message.SocketID(),
                          message.IP(),
                          account,
                          std::move(validation.worldServer),
                          validation.hasMatrix);
    switch (m_Context.PrepareEnter(checked)) {
    case PrepareEnterOutcome::Continue:
        m_Context.EnterGame(checked, m_Queue.IsInNoQueueList(checked.Account()));
        return std::nullopt;
    case PrepareEnterOutcome::Finished:
        return std::nullopt;
    case PrepareEnterOutcome::MatrixRegistrationRequired:
        if (const auto error = m_Queue.MatrixRegister(m_Context, checked)) {
            return LogMessageError{
                .kind = LogMessageErrorKind::MatrixRandom,
                .detail = error->detail,
            };
        }
        return std::nullopt;
    }
    return std::nullopt;
}

std::optional<LogMessageError> LogMessageHandler::OnValidCodeRefresh(
    LoginNet::CMessage& message)
{
    const auto account = GetBoundedString(message, kAccountLimit);
    switch (m_Queue.CheckMsgInfo(account, message.SocketID())) {
    case CheckMessageInfo::Success: {
        CValidCode validCode;
        if (auto error = CValidCode::Generate(std::filesystem::path{"."}, validCode)) {
            return LogMessageError{
                .kind = LogMessageErrorKind::ValidCodeGeneration,
                .detail = error->detail,
                .validCodeError = std::move(error),
            };
        }
        m_Queue.ChangeValidCode(account, validCode.ValidCode());

        LoginNet::CMessage response(kLoginResponseMessageType);
        response.Base().Add(static_cast<char>('J'));
        AddLegacyString(response, account);
        response.Base().Add(static_cast<std::int32_t>(kValidCodeBitmapLength));
        response.Base().Add(validCode.Bitmap().data(),
                            static_cast<std::int32_t>(kValidCodeBitmapLength));
        m_Context.SendToClient(response, message.SocketID());
        return std::nullopt;
    }
    case CheckMessageInfo::Frequent:
        SendCode(m_Context, message.SocketID(), static_cast<std::uint8_t>('O'));
        return std::nullopt;
    case CheckMessageInfo::Missing:
        SendCode(m_Context, message.SocketID(), static_cast<std::uint8_t>('P'));
        return std::nullopt;
    case CheckMessageInfo::SocketMismatch:
        return std::nullopt;
    }
    return std::nullopt;
}

void LogMessageHandler::OnWorldListRequest(LoginNet::CMessage& message)
{
    const auto account = GetBoundedString(message, kExtendedAccountLimit);
    if (account.empty()) {
        return;
    }

    LoginNet::CMessage response(kWorldListResponseMessageType);
    AddLegacyString(response, account);
    m_Context.AddWorldInfoToMsg(response, account);
    m_Context.SendToClient(response, message.SocketID());
}

void LogMessageHandler::OnSyntheticDisconnect(LoginNet::CMessage& message)
{
    const auto account = GetBoundedString(message, kExtendedAccountLimit);
    static_cast<void>(m_Context.ClearLoginCdkey(account));
    m_Context.QuitClientByCdkey(account);

    if (m_Context.FindCdkey(account) != -1) {
        return;
    }

    const auto worldServer = m_Context.LoginCdkeyWorldServer(account);
    const std::int32_t worldId = worldServer ? m_Context.WorldIDByName(*worldServer) : -1;
    if (worldId != -1) {
        LoginNet::CMessage notice(kWorldClientLostMessageType);
        AddLegacyString(notice, account);
        m_Context.SendToWorld(notice, worldId);
    }

    m_Context.AccountLeaveLog(account);
    m_Context.ClearCdkey(account);
    static_cast<void>(m_Queue.OnClientLost(account));
}

void LogMessageHandler::OnWorldLoginResult(LoginNet::CMessage& message)
{
    const std::uint8_t status = static_cast<std::uint8_t>(message.Base().GetChar());
    const auto account = GetBoundedString(message, kExtendedAccountLimit);
    const std::int32_t worldId = message.MapID();

    if (status == kWorldLoginSuccessStatus) {
        static_cast<void>(m_Context.AddCdkey(account, worldId));
        static_cast<void>(GetBoundedString(message, kRoleFieldLimit));
        static_cast<void>(message.Base().GetLong());
        const auto roleName = GetBoundedString(message, kRoleFieldLimit);
        const std::uint8_t roleLevel = static_cast<std::uint8_t>(message.Base().GetChar());
        m_Context.RoleEnterLog(account, roleName, roleLevel, worldId & 0xFF);
    }

    RelayWorldResponse(message, account, kLoginResultResponseMessageType);
}

void LogMessageHandler::OnWorldPlayerBaseResponse(LoginNet::CMessage& message)
{
    static_cast<void>(message.Base().GetChar());
    const auto account = GetBoundedString(message, kExtendedAccountLimit);
    RelayWorldResponse(message, account, kPlayerBaseResponseMessageType);
}

void LogMessageHandler::OnWorldDeleteRoleResponse(LoginNet::CMessage& message)
{
    static_cast<void>(message.Base().GetChar());
    static_cast<void>(message.Base().GetLong());
    const auto account = GetBoundedString(message, kExtendedAccountLimit);
    RelayWorldResponse(message, account, kDeleteRoleResponseMessageType);
}

void LogMessageHandler::OnWorldRestoreRoleResponse(LoginNet::CMessage& message)
{
    static_cast<void>(message.Base().GetChar());
    static_cast<void>(message.Base().GetLong());
    const auto account = GetBoundedString(message, kExtendedAccountLimit);
    RelayWorldResponse(message, account, kRestoreRoleResponseMessageType);
}

void LogMessageHandler::OnWorldCreateRoleResponse(LoginNet::CMessage& message)
{
    static_cast<void>(message.Base().GetChar());
    const auto account = GetBoundedString(message, kExtendedAccountLimit);
    RelayWorldResponse(message, account, kCreateRoleResponseMessageType);
}

void LogMessageHandler::OnWorldLeaveResult(LoginNet::CMessage& message)
{
    const auto account = GetBoundedString(message, kExtendedAccountLimit);
    m_Context.ClearCdkey(account);
    static_cast<void>(GetBoundedString(message, kRoleFieldLimit));
    static_cast<void>(message.Base().GetChar());
    m_Context.LeaveLog(account);
}

void LogMessageHandler::OnWorldResponse1FF07(LoginNet::CMessage& message)
{
    const auto account = GetBoundedString(message, kExtendedAccountLimit);
    RelayWorldResponse(message, account, kResponseAF508MessageType);
}

void LogMessageHandler::RelayWorldResponse(LoginNet::CMessage& message,
                                           std::span<const std::uint8_t> account,
                                           std::int32_t responseType)
{
    message.SetMessageType(responseType);
    m_Context.SendToClientCdkey(message, account);
}
}
