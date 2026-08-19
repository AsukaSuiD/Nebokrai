#include "gasthread.h"

#include "gasoperator.h"
#include "../../nets/netlogin/message.h"

#include <openssl/evp.h>

#include <algorithm>
#include <array>
#include <chrono>
#include <ctime>
#include <string_view>
#include <thread>
#include <tuple>
#include <utility>

namespace Login
{
namespace
{
constexpr std::int32_t kLoginResponseMessageType = 0x000AF501;
constexpr std::size_t kLegacySignSourceBufferSize = 0x200U;
constexpr std::size_t kLegacyFormContentBufferSize = 0x400U;
constexpr std::string_view kNicknameKey = "\"nickname\"";
constexpr std::string_view kSignSalt = "DaYeZaiCi";
constexpr std::array<char, 16> kLowerHex{
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
};

std::span<const std::uint8_t> LegacyCStringPrefix(std::span<const std::uint8_t> value)
{
    const auto terminator = std::find(value.begin(), value.end(), std::uint8_t{0});
    return value.first(static_cast<std::size_t>(std::distance(value.begin(), terminator)));
}

std::string LegacyString(std::span<const std::uint8_t> value)
{
    const auto prefix = LegacyCStringPrefix(value);
    if (prefix.empty()) {
        return {};
    }
    return std::string(reinterpret_cast<const char*>(prefix.data()), prefix.size());
}

void AddLegacyString(LoginNet::CMessage& message, std::span<const std::uint8_t> value)
{
    const auto prefix = LegacyCStringPrefix(value);
    if (!prefix.empty()) {
        message.Base().Add(prefix.data(), static_cast<std::int32_t>(prefix.size()));
    }
    message.Base().Add(std::uint8_t{0});
}

std::optional<LocalDateTime> CurrentLocalDateTime()
{
    const auto now = std::chrono::system_clock::now();
    const std::time_t time = std::chrono::system_clock::to_time_t(now);
    std::tm local{};
#if defined(_WIN32)
    if (::localtime_s(&local, &time) != 0) {
        return std::nullopt;
    }
#else
    if (::localtime_r(&time, &local) == nullptr) {
        return std::nullopt;
    }
#endif
    const auto millis = std::chrono::duration_cast<std::chrono::milliseconds>(
                            now.time_since_epoch())
                            .count();
    return LocalDateTime{
        .year = static_cast<std::uint16_t>(local.tm_year + 1900),
        .month = static_cast<std::uint16_t>(local.tm_mon + 1),
        .day = static_cast<std::uint16_t>(local.tm_mday),
        .hour = static_cast<std::uint16_t>(local.tm_hour),
        .minute = static_cast<std::uint16_t>(local.tm_min),
        .second = static_cast<std::uint16_t>(local.tm_sec),
        .milliseconds = static_cast<std::uint16_t>(
            static_cast<std::uint64_t>(millis) % 1000ULL),
    };
}

bool EarlierThan(const LocalDateTime& left, const LocalDateTime& right)
{
    return std::tie(left.year, left.month, left.day, left.hour, left.minute,
                    left.second, left.milliseconds) <
           std::tie(right.year, right.month, right.day, right.hour, right.minute,
                    right.second, right.milliseconds);
}

std::string HexLower(std::span<const std::uint8_t, 16> bytes)
{
    std::string result(32U, '\0');
    for (std::size_t index = 0; index < bytes.size(); ++index) {
        const std::uint8_t value = bytes[index];
        result[index * 2U] = kLowerHex[(value >> 4U) & 0x0FU];
        result[index * 2U + 1U] = kLowerHex[value & 0x0FU];
    }
    return result;
}

std::optional<std::array<std::uint8_t, 16>> Md5(std::string_view input)
{
    std::array<std::uint8_t, 16> digest{};
    unsigned int digestLength = 0;
    if (EVP_Digest(input.data(), input.size(), digest.data(), &digestLength,
                   EVP_md5(), nullptr) != 1 ||
        digestLength != digest.size()) {
        return std::nullopt;
    }
    return digest;
}

void UpperAscii(std::string& value)
{
    for (char& character : value) {
        if (character >= 'a' && character <= 'z') {
            character = static_cast<char>(character - 'a' + 'A');
        }
    }
}

GasThreadError Error(GasThreadErrorKind kind,
                     std::span<const std::uint8_t> account,
                     std::string detail,
                     std::size_t actualLength = 0U)
{
    return GasThreadError{
        .kind = kind,
        .account = {account.begin(), account.end()},
        .actualLength = actualLength,
        .detail = std::move(detail),
    };
}

void SendLoginCode(IGasThreadContext& context,
                   std::int32_t socketId,
                   std::uint8_t code)
{
    LoginNet::CMessage response(kLoginResponseMessageType);
    response.Base().Add(static_cast<char>(code));
    context.SendToClient(response, socketId);
}

bool SendGasFailureForState(IGasThreadContext& context,
                            std::int32_t socketId,
                            std::int32_t state)
{
    LoginNet::CMessage response(kLoginResponseMessageType);
    switch (state) {
    case 100:
        response.Base().Add(static_cast<char>(0x07));
        break;
    case -11:
    case -10:
        response.Base().Add(static_cast<char>('Y'));
        response.Base().Add(static_cast<char>(0x01));
        break;
    case -8:
        response.Base().Add(static_cast<char>('Y'));
        response.Base().Add(static_cast<char>(0x00));
        break;
    case -7:
    case -4:
        response.Base().Add(static_cast<char>(0x07));
        break;
    case -6:
    case -3:
        response.Base().Add(static_cast<char>(0x05));
        break;
    case -5:
        response.Base().Add(static_cast<char>('T'));
        break;
    case -2:
    case -1:
        response.Base().Add(static_cast<char>('?'));
        break;
    default:
        return false;
    }
    context.SendToClient(response, socketId);
    return true;
}
}

CGasThread::CGasThread(CLoginQueue& queue, IGasThreadContext& context)
    : m_Queue(queue), m_Context(context)
{
}

GasRunReport CGasThread::RunOnce()
{
    GasRunReport report;
    while (auto quest = m_Queue.PopGasQueue()) {
        ++report.processed;
        if (auto error = ProcessQuest(*quest, report)) {
            report.errors.push_back(std::move(*error));
        }
    }
    return report;
}

void CGasThread::Run()
{
    for (;;) {
        static_cast<void>(RunOnce());
        // VERIFIED_ASSEMBLY 0x0042187B: Sleep(10) только после полного drain.
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
}

std::int32_t CGasThread::GetNickNameFromStrs(std::string_view response,
                                             std::string& nickname) const
{
    nickname.clear();
    if (response.size() > 0xFFFFU || kNicknameKey.size() > response.size()) {
        return -1;
    }

    // VERIFIED_ASSEMBLY 0x00420FA0: исходный поиск идёт только при
    // offset < response_len - key_len. Совпадение в самой последней возможной
    // позиции намеренно НЕ рассматривается.
    const std::size_t searchSpan = response.size() - kNicknameKey.size();
    if (searchSpan == 0U) {
        return -1;
    }

    std::size_t found = std::string_view::npos;
    for (std::size_t offset = 0; offset < searchSpan; ++offset) {
        if (response.substr(offset, kNicknameKey.size()) == kNicknameKey) {
            found = offset;
            break;
        }
    }
    if (found == std::string_view::npos) {
        return -1;
    }

    const std::size_t afterKey = found + kNicknameKey.size();
    // Helper 0x00420F50 получает afterKey+1, ожидая JSON-ish :"value".
    // Невалидный хвост безопасно сводим к тому же parser-failure -1, не читая OOB.
    if (afterKey + 2U > response.size()) {
        return -1;
    }
    const std::string_view quoted = response.substr(afterKey + 1U);
    if (quoted.size() < 2U) {
        return -1;
    }
    if (quoted[1] == '"') {
        return 0;
    }

    for (std::size_t index = 1U; index < quoted.size(); ++index) {
        if (quoted[index] == '"') {
            const std::size_t length = index - 1U;
            nickname.assign(quoted.substr(1U, length));
            return static_cast<std::int32_t>(length);
        }
    }
    return -1;
}

std::int32_t CGasThread::AnalysisRet(std::string_view response,
                                     std::string& nickname) const
{
    if (response.empty()) {
        return 100;
    }

    // VERIFIED caller-count=1: CheckAcc передаёт CMyWinInet::m_strData[1024],
    // который ctor/Close полностью обнуляют. Поэтому при strlen 1..11 прямые
    // чтения [10]/[11] в EXE дают 0, а не неизвестную память.
    const char stateMarker = response.size() > 10U ? response[10] : '\0';
    const char stateDigit = response.size() > 11U ? response[11] : '\0';
    if (stateMarker == '-') {
        switch (stateDigit) {
        case '1': return -1;
        case '2': return -2;
        case '3': return -3;
        case '4': return -4;
        case '5': return -5;
        case '7': return -7;
        case '8': return -8;
        default: return -6;
        }
    }

    const std::int32_t parsed = GetNickNameFromStrs(response, nickname);
    if (parsed == 0) {
        return -10;
    }
    if (parsed == -1) {
        return -11;
    }
    return 0;
}

std::optional<std::string> CGasThread::MD5vec2str(
    std::span<const std::uint8_t> digest,
    GasThreadError& error,
    std::span<const std::uint8_t> account) const
{
    if (digest.size() < 16U) {
        error = Error(GasThreadErrorKind::PasswordDigestTooShort,
                      account,
                      "GAS MD5vec2str requires the first 16 digest bytes",
                      digest.size());
        return std::nullopt;
    }
    std::array<std::uint8_t, 16> first{};
    std::copy_n(digest.begin(), first.size(), first.begin());
    return HexLower(std::span<const std::uint8_t, 16>(first));
}

std::optional<std::string> CGasThread::FormContent(
    const CLoginQueue::QuestCdkey& quest,
    GasThreadError& error) const
{
    auto passwordHex = MD5vec2str(quest.passwordDigest, error, quest.account);
    if (!passwordHex) {
        return std::nullopt;
    }

    const std::string account = LegacyString(quest.account);
    std::string signSource;
    signSource.reserve(account.size() + passwordHex->size() + kSignSalt.size() + 2U);
    signSource.append(account);
    signSource.push_back('|');
    signSource.append(*passwordHex);
    signSource.push_back('|');
    signSource.append(kSignSalt);
    if (signSource.size() >= kLegacySignSourceBufferSize) {
        error = Error(GasThreadErrorKind::SignSourceTooLong,
                      quest.account,
                      "GAS sign source does not fit legacy char[512]",
                      signSource.size());
        return std::nullopt;
    }

    const auto digest = Md5(signSource);
    if (!digest) {
        error = Error(GasThreadErrorKind::Md5Unavailable,
                      quest.account,
                      "OpenSSL EVP could not compute legacy MD5");
        return std::nullopt;
    }
    std::string hash = HexLower(std::span<const std::uint8_t, 16>(*digest));
    if (m_Context.VerificationSignUpper() == 1) {
        UpperAscii(hash);
    }

    std::string content;
    content.reserve(account.size() + passwordHex->size() + hash.size() + 25U);
    content.append("username=");
    content.append(account);
    content.append("&password=");
    content.append(*passwordHex);
    content.append("&hash=");
    content.append(hash);
    if (content.size() >= kLegacyFormContentBufferSize) {
        error = Error(GasThreadErrorKind::FormContentTooLong,
                      quest.account,
                      "GAS POST content does not fit legacy char[1024]",
                      content.size());
        return std::nullopt;
    }
    return content;
}

CGasThread::CheckAccResult CGasThread::CheckAcc(CLoginQueue::QuestCdkey& quest)
{
    const std::string verificationAddress = m_Context.VerificationAddress();
    char* response = nullptr;

    if (m_myWinInet.Init(verificationAddress.c_str()) != 0) {
        GasThreadError contentError;
        auto content = FormContent(quest, contentError);
        if (!content) {
            m_myWinInet.Close();
            return CheckAccResult{.error = std::move(contentError)};
        }
        if (m_myWinInet.Send(content->c_str()) != 0) {
            response = m_myWinInet.Recv();
        }
    }

    const auto transportBoundary = [&]() -> std::optional<GasThreadError> {
        const auto& error = m_myWinInet.LastError();
        if (!error) {
            return std::nullopt;
        }
        switch (error->kind) {
        case MyWinInetErrorKind::UrlTooLong:
        case MyWinInetErrorKind::ResponseTooLarge:
        case MyWinInetErrorKind::TlsPeerVerificationLegacyRetryBoundary:
            return Error(GasThreadErrorKind::TransportCompatibilityBoundary,
                         quest.account,
                         error->detail);
        case MyWinInetErrorKind::CurlGlobal:
        case MyWinInetErrorKind::InvalidUrl:
        case MyWinInetErrorKind::CurlInit:
        case MyWinInetErrorKind::CurlOptions:
        case MyWinInetErrorKind::Transfer:
            return std::nullopt;
        }
        return std::nullopt;
    };

    std::string nickname;
    if (response == nullptr) {
        if (auto boundary = transportBoundary()) {
            m_myWinInet.Close();
            return CheckAccResult{.error = std::move(boundary)};
        }
        m_myWinInet.Close();
        return CheckAccResult{.state = 100};
    }

    const std::string_view responseView(response);
    const std::int32_t state = AnalysisRet(responseView, nickname);
    if (state == 0) {
        quest.nickname.assign(nickname.begin(), nickname.end());
        quest.account.assign(nickname.begin(), nickname.end());
    }
    m_myWinInet.Close();
    return CheckAccResult{.state = state};
}

std::optional<GasThreadError> CGasThread::ProcessQuest(
    CLoginQueue::QuestCdkey& quest,
    GasRunReport& report)
{
    if (!quest.worldServer.empty()) {
        TagPwdChecked checked(quest.socketId, quest.clientIp, quest.account,
                              quest.worldServer, false);
        const PrepareEnterOutcome outcome = m_Context.PrepareEnter(checked);
        if (outcome == PrepareEnterOutcome::Continue) {
            m_Context.EnterGame(checked, m_Queue.IsInNoQueueList(checked.Account()));
            ++report.forwardedToEnter;
            return std::nullopt;
        }

        // VERIFIED_ASSEMBLY 0x0042183C..0x00421849: только non-zero
        // PrepareEnter path делает эту странную GetInstance/ReleaseInstance пару.
        static_cast<void>(CGasOperator::GetInstance());
        CGasOperator::ReleaseInstance();
        return std::nullopt;
    }

    CheckAccResult check = CheckAcc(quest);
    if (check.error) {
        return std::move(check.error);
    }
    if (!check.state) {
        return Error(GasThreadErrorKind::CheckAccInvariant,
                     quest.account,
                     "CheckAcc produced neither GAS state nor technical error");
    }
    if (*check.state != 0) {
        if (SendGasFailureForState(m_Context, quest.socketId, *check.state)) {
            ++report.gasRejected;
        } else {
            ++report.droppedUnknownState;
        }
        return std::nullopt;
    }

    if (!m_Context.HasRsCdKeyOwner()) {
        return Error(GasThreadErrorKind::DatabaseOwnerMissing,
                     quest.account,
                     "CRsCDKey owner for successful GAS authentication is missing");
    }

    const double banVariantTime = m_Context.BanVariantTime(quest.account);
    if (banVariantTime != 0.0) {
        // VERIFIED_ASSEMBLY 0x004218F4..0x00421976: GetLocalTime идёт раньше
        // VariantTimeToSystemTime, как и в OnQuestCdkey.
        const auto now = CurrentLocalDateTime();
        if (!now) {
            return Error(GasThreadErrorKind::LocalTimeUnavailable,
                         quest.account,
                         "system localtime failed in GAS worker");
        }
        const auto banTime = m_Context.DecodeVariantTime(banVariantTime);
        if (!banTime) {
            return Error(GasThreadErrorKind::VariantTimeConversionFailed,
                         quest.account,
                         "OLE DATE ban_time was not decoded in GAS worker");
        }
        if (EarlierThan(*now, *banTime)) {
            LoginNet::CMessage response(kLoginResponseMessageType);
            response.Base().Add(static_cast<char>(0x10));
            response.Base().Add(static_cast<char>(0));
            response.Base().Add(static_cast<std::int16_t>(banTime->year));
            response.Base().Add(static_cast<std::int16_t>(banTime->month));
            response.Base().Add(static_cast<std::int16_t>(banTime->day));
            response.Base().Add(static_cast<std::int16_t>(banTime->hour));
            response.Base().Add(static_cast<std::int16_t>(banTime->minute));
            m_Context.SendToClient(response, quest.socketId);
            ++report.gasRejected;
            return std::nullopt;
        }
    }

    if (!m_Context.IpIsAllowed(quest.clientIp)) {
        SendLoginCode(m_Context, quest.socketId, 0x12U);
        ++report.gasRejected;
        return std::nullopt;
    }
    if (m_Context.IpIsForbidden(quest.clientIp)) {
        SendLoginCode(m_Context, quest.socketId, 0x12U);
        ++report.gasRejected;
        return std::nullopt;
    }
    if (!m_Context.IsBetweenIp(quest.account, quest.clientIp)) {
        SendLoginCode(m_Context, quest.socketId, 0x11U);
        ++report.gasRejected;
        return std::nullopt;
    }

    LoginNet::CMessage response(kLoginResponseMessageType);
    response.Base().Add(static_cast<char>(0x02));
    AddLegacyString(response, quest.account);
    m_Context.AddWorldInfoToMsg(response, quest.account);
    m_Context.SendToClient(response, quest.socketId);

    m_Queue.PushBackPwdChecked(
        TagPwdChecked(quest.socketId, quest.clientIp, quest.account,
                      quest.worldServer, false),
        [this](std::span<const std::uint8_t> duplicateAccount) {
            m_Context.KickOut(duplicateAccount);
        });

    CGasOperator* gasOperator = CGasOperator::GetInstance();
    if (gasOperator == nullptr) {
        return Error(GasThreadErrorKind::GasOperatorUnavailable,
                     quest.account,
                     "CGasOperator allocation failed after GAS success");
    }
    const std::string clientIp = gasOperator->GetIP(quest.clientIp);

    GasThreadError passwordError;
    auto passwordHex = MD5vec2str(quest.passwordDigest, passwordError, quest.account);
    if (!passwordHex) {
        return passwordError;
    }

    const std::string nickname = LegacyString(quest.nickname);
    static_cast<void>(m_Context.ExecuteProce(nickname, clientIp,
                                             passwordHex->data(), 0));
    ++report.authenticated;
    return std::nullopt;
}
}
