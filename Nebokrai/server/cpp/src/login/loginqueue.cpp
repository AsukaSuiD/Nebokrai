#include "loginqueue.h"

#include "authmanager.h"
#include "applogin/validcode.h"
#include "../nets/netlogin/message.h"

#include <algorithm>
#include <bit>
#include <cerrno>
#include <chrono>
#include <ctime>
#include <exception>
#include <fstream>
#include <iterator>
#include <random>
#include <string_view>
#include <system_error>
#include <tuple>
#include <utility>

#if defined(__linux__)
#include <sys/random.h>
#endif

namespace Login
{
namespace
{
constexpr std::int32_t kLoginResponseMessageType = 0x000AF501;
constexpr std::int32_t kPlayerDataRejectMessageType = 0x000AF503;
constexpr std::int32_t kQueuePositionMessageType = 0x000AF507;
constexpr std::size_t kNoQueueAccountBufferSize = 0x100U;

std::span<const std::uint8_t> LegacyCStringPrefix(std::span<const std::uint8_t> value)
{
    const auto terminator = std::find(value.begin(), value.end(), std::uint8_t{0});
    return value.first(static_cast<std::size_t>(std::distance(value.begin(), terminator)));
}

std::vector<std::uint8_t> OwnedLegacyString(std::span<const std::uint8_t> value)
{
    const auto prefix = LegacyCStringPrefix(value);
    return {prefix.begin(), prefix.end()};
}

bool BytesEqual(std::span<const std::uint8_t> left,
                std::span<const std::uint8_t> right)
{
    return left.size() == right.size() &&
           std::equal(left.begin(), left.end(), right.begin());
}

bool AsciiEqualInsensitive(std::string_view left, std::string_view right)
{
    if (left.size() != right.size()) {
        return false;
    }
    for (std::size_t index = 0; index < left.size(); ++index) {
        auto l = static_cast<unsigned char>(left[index]);
        auto r = static_cast<unsigned char>(right[index]);
        if (l >= 'A' && l <= 'Z') l = static_cast<unsigned char>(l - 'A' + 'a');
        if (r >= 'A' && r <= 'Z') r = static_cast<unsigned char>(r - 'A' + 'a');
        if (l != r) {
            return false;
        }
    }
    return true;
}

std::optional<std::filesystem::path> ResolveWindowsAsset(
    const std::filesystem::path& directory,
    std::string_view requested,
    std::error_code& error)
{
    error.clear();
    const auto direct = directory / std::string(requested);
    if (std::filesystem::exists(direct, error)) {
        return direct;
    }
    if (error) {
        return std::nullopt;
    }

    std::filesystem::directory_iterator iterator(directory, error);
    const std::filesystem::directory_iterator end;
    while (!error && iterator != end) {
        const auto filename = iterator->path().filename().string();
        if (AsciiEqualInsensitive(filename, requested)) {
            return iterator->path();
        }
        iterator.increment(error);
    }
    return std::nullopt;
}

bool IsAsciiWhitespace(std::uint8_t byte)
{
    return byte == ' ' || byte == '\t' || byte == '\n' || byte == '\r' ||
           byte == '\f' || byte == '\v';
}

void LowerAscii(std::vector<std::uint8_t>& value)
{
    for (auto& byte : value) {
        if (byte >= 'A' && byte <= 'Z') {
            byte = static_cast<std::uint8_t>(byte - 'A' + 'a');
        }
    }
}

std::uint32_t LegacyTickMs()
{
    return AuthManager::LegacyTickMs();
}

std::optional<MatrixRegisterError> RandomWord(std::uint32_t& value)
{
#if defined(__linux__)
    auto* output = reinterpret_cast<std::uint8_t*>(&value);
    std::size_t filled = 0;
    while (filled < sizeof(value)) {
        const ssize_t received = ::getrandom(output + filled, sizeof(value) - filled, 0);
        if (received > 0) {
            filled += static_cast<std::size_t>(received);
            continue;
        }
        if (received < 0 && errno == EINTR) {
            continue;
        }
        return MatrixRegisterError{
            .kind = MatrixRegisterErrorKind::Random,
            .detail = "getrandom не вернул 32-битное значение matrix",
        };
    }
    return std::nullopt;
#else
    try {
        static thread_local std::random_device source;
        value = static_cast<std::uint32_t>(source());
        return std::nullopt;
    } catch (const std::exception& exception) {
        return MatrixRegisterError{
            .kind = MatrixRegisterErrorKind::Random,
            .detail = exception.what(),
        };
    }
#endif
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
    const auto millisecond = static_cast<std::uint16_t>(
        static_cast<std::uint64_t>(millis) % 1000ULL);
    return LocalDateTime{
        .year = static_cast<std::uint16_t>(local.tm_year + 1900),
        .month = static_cast<std::uint16_t>(local.tm_mon + 1),
        .day = static_cast<std::uint16_t>(local.tm_mday),
        .hour = static_cast<std::uint16_t>(local.tm_hour),
        .minute = static_cast<std::uint16_t>(local.tm_min),
        .second = static_cast<std::uint16_t>(local.tm_sec),
        .milliseconds = millisecond,
    };
}

bool EarlierThan(const LocalDateTime& left, const LocalDateTime& right)
{
    return std::tie(left.year, left.month, left.day, left.hour, left.minute,
                    left.second, left.milliseconds) <
           std::tie(right.year, right.month, right.day, right.hour, right.minute,
                    right.second, right.milliseconds);
}

bool IsAsciiNumeric(std::span<const std::uint8_t> value)
{
    return std::all_of(value.begin(), value.end(), [](std::uint8_t byte) {
        return byte >= '0' && byte <= '9';
    });
}

std::optional<std::vector<std::uint8_t>> PasswordDigestHex(
    std::span<const std::uint8_t> digest)
{
    if (digest.size() < 16U) {
        return std::nullopt;
    }
    static constexpr std::array<std::uint8_t, 16> kHex{
        '0', '1', '2', '3', '4', '5', '6', '7',
        '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
    };
    std::vector<std::uint8_t> result(32U);
    for (std::size_t index = 0; index < 16U; ++index) {
        const std::uint8_t byte = digest[index];
        result[index * 2U] = kHex[(byte >> 4U) & 0x0FU];
        result[index * 2U + 1U] = kHex[byte & 0x0FU];
    }
    return result;
}

void SendLoginCode(ILoginQueueContext& context,
                   std::int32_t socketId,
                   std::uint8_t code)
{
    LoginNet::CMessage response(kLoginResponseMessageType);
    response.Base().Add(static_cast<char>(code));
    context.SendToClient(response, socketId);
}

void SendQueuePosition(ILoginQueueContext& context,
                       std::int32_t socketId,
                       std::int32_t position)
{
    LoginNet::CMessage response(kQueuePositionMessageType);
    response.Base().Add(position);
    context.SendToClient(response, socketId);
}

template <typename Queue>
bool EraseFirstAccount(Queue& queue, std::span<const std::uint8_t> account)
{
    const auto found = std::find_if(queue.begin(), queue.end(), [&](const auto& pending) {
        return BytesEqual(pending.account, account);
    });
    if (found == queue.end()) {
        return false;
    }
    queue.erase(found);
    return true;
}

template <typename Map>
std::size_t EraseFirstAccountFromEachWorld(Map& queues,
                                           std::span<const std::uint8_t> account)
{
    std::size_t removed = 0;
    for (auto& [world, queue] : queues) {
        static_cast<void>(world);
        if (EraseFirstAccount(queue, account)) {
            ++removed;
        }
    }
    return removed;
}
}

TagPwdChecked::TagPwdChecked(std::int32_t socketId,
                             std::uint32_t clientIp,
                             std::vector<std::uint8_t> account,
                             std::vector<std::uint8_t> worldServer,
                             bool hasMatrix)
    : m_ClientIP(clientIp),
      m_SocketID(socketId),
      m_Account(std::move(account)),
      m_WorldServer(std::move(worldServer)),
      m_HasMatrix(hasMatrix)
{
}

std::int32_t TagPwdChecked::SocketID() const noexcept
{
    return m_SocketID;
}

std::uint32_t TagPwdChecked::ClientIP() const noexcept
{
    return m_ClientIP;
}

std::span<const std::uint8_t> TagPwdChecked::Account() const noexcept
{
    return m_Account;
}

std::span<const std::uint8_t> TagPwdChecked::WorldServer() const noexcept
{
    return m_WorldServer;
}

bool TagPwdChecked::HasMatrix() const noexcept
{
    return m_HasMatrix;
}

CLoginQueue::CLoginQueue(const std::filesystem::path& runtimeDirectory)
{
    static_cast<void>(LoadNoQueueCdkeyList(runtimeDirectory));
}

void CLoginQueue::OnInitial(std::uint32_t intervalMs,
                            std::uint32_t sendMessageIntervalMs,
                            std::uint32_t worldMaxPlayers)
{
    std::lock_guard guard(m_SetupMutex);
    m_IntervalMs = intervalMs;
    m_SendMessageIntervalMs = sendMessageIntervalMs;
    m_WorldMaxPlayers = std::bit_cast<std::int32_t>(worldMaxPlayers);
    if (m_WorldCount == 0U) {
        m_WorldCount = 1U;
    }

    const std::uint32_t now = LegacyTickMs();
    m_WorldQueueTime = now + m_IntervalMs;
    m_LogQueueTime = now + (m_IntervalMs / m_WorldCount);
}

void CLoginQueue::SetWorldCount(std::uint32_t worldCount)
{
    std::lock_guard guard(m_SetupMutex);
    m_WorldCount = std::bit_cast<std::int32_t>(worldCount);
}

LoginQueueRunReport CLoginQueue::Run(ILoginQueueContext& context,
                                     AuthManager& authManager)
{
    LoginQueueRunReport report;
    const auto recordQuestError = [&](const QuestCdkey& quest,
                                      std::optional<QuestCdkeyError> error) {
        if (!error) {
            return;
        }
        report.questCdkeyErrors.push_back(RunQuestCdkeyError{
            .account = quest.account,
            .error = std::move(*error),
        });
    };

    // ПОДТВЕРЖДЕНО ДИЗАССЕМБЛИРОВАНИЕМ/АССЕМБЛЕРОМ 0x0041D51B..0x0041D56F:
    // m_GasQuest (+0x64/+0x68) не очищается. Если list непуст, после его
    // полного прохода EXE ошибочно clear-ит m_NoQueueQuestCdkey (+0x34).
    if (!m_GasQuest.empty()) {
        for (const QuestCdkey& quest : m_GasQuest) {
            recordQuestError(quest, OnQuestCdkey(context, authManager, quest));
        }
        std::lock_guard guard(m_QuestCdkeyMutex);
        m_NoQueueQuestCdkey.clear();
    }

    // No-queue CD-key обрабатываются целиком без cadence.
    {
        std::lock_guard guard(m_QuestCdkeyMutex);
        if (!m_NoQueueQuestCdkey.empty()) {
            for (const QuestCdkey& quest : m_NoQueueQuestCdkey) {
                recordQuestError(quest, OnQuestCdkey(context, authManager, quest));
            }
            m_NoQueueQuestCdkey.clear();
        }
    }

    // No-queue player-list/data тоже полностью дренируются до первого DVar9.
    {
        std::lock_guard guard(m_PlayerQuestMutex);
        if (!m_NoQueueQuestPlayerList.empty()) {
            for (auto& [world, quests] : m_NoQueueQuestPlayerList) {
                static_cast<void>(world);
                for (const QuestPlayerList& quest : quests) {
                    if (context.L2WPlayerBaseSend(quest.worldServer, quest.account)) {
                        context.SetLoginCdkeyWorldServer(quest.account, quest.worldServer);
                    }
                }
            }
            m_NoQueueQuestPlayerList.clear();
        }
        if (!m_NoQueueQuestPlayerData.empty()) {
            for (auto& [world, quests] : m_NoQueueQuestPlayerData) {
                static_cast<void>(world);
                for (const QuestPlayerData& quest : quests) {
                    OnQuestPlayerData(context, quest);
                }
            }
            m_NoQueueQuestPlayerData.clear();
        }
    }

    // DVar9 снимается ОДИН раз и далее используется log cadence, World cadence
    // и всеми queue-position notices. Поздние timeout-группы берут новые ticks.
    const std::uint32_t queueNow = LegacyTickMs();

    bool doLogQueue = false;
    {
        std::lock_guard guard(m_SetupMutex);
        if (m_LogQueueTime <= queueNow) {
            doLogQueue = true;
            m_LogQueueTime = m_WorldCount == 0U
                ? queueNow + 1000U
                : queueNow +
                    (m_IntervalMs / std::bit_cast<std::uint32_t>(m_WorldCount));
        }
    }
    if (doLogQueue) {
        std::lock_guard guard(m_QuestCdkeyMutex);
        if (!m_QuestCdkey.empty()) {
            QuestCdkey& quest = m_QuestCdkey.front();
            recordQuestError(quest, OnQuestCdkey(context, authManager, quest));
            // Исходник удаляет front после void OnQuestCdkey независимо от
            // результата внутренних send/routing операций.
            m_QuestCdkey.pop_front();
        }
    }

    report.pwdChecked = HandlePwdChecked(context);

    bool doWorldQueue = false;
    std::int32_t worldMaxPlayers = 0;
    {
        std::lock_guard guard(m_SetupMutex);
        if (m_WorldQueueTime <= queueNow) {
            doWorldQueue = true;
            m_WorldQueueTime = queueNow + m_IntervalMs;
        }
        worldMaxPlayers = m_WorldMaxPlayers;
    }
    if (doWorldQueue) {
        std::lock_guard guard(m_PlayerQuestMutex);
        for (auto& [world, quests] : m_QuestPlayerList) {
            const std::int32_t currentPlayers = context.LoginWorldPlayerNumByName(world);
            if (currentPlayers < worldMaxPlayers && !quests.empty()) {
                const QuestPlayerList& quest = quests.front();
                if (context.L2WPlayerBaseSend(quest.worldServer, quest.account)) {
                    context.SetLoginCdkeyWorldServer(quest.account, quest.worldServer);
                }
                quests.pop_front();
            }
        }
        for (auto& [world, quests] : m_QuestPlayerData) {
            static_cast<void>(world);
            if (!quests.empty()) {
                OnQuestPlayerData(context, quests.front());
                quests.pop_front();
            }
        }
    }

    const std::uint32_t sendInterval = SendMessageIntervalMs();
    {
        std::lock_guard guard(m_QuestCdkeyMutex);
        std::int32_t position = 1;
        for (QuestCdkey& quest : m_QuestCdkey) {
            if (quest.sendMessageTime <= queueNow) {
                quest.sendMessageTime = queueNow + sendInterval;
                SendQueuePosition(context, quest.socketId, position);
                ++report.queuePositionMessages;
            }
            position = static_cast<std::int32_t>(
                std::bit_cast<std::uint32_t>(position) + 1U);
        }
    }
    {
        std::lock_guard guard(m_PlayerQuestMutex);
        for (auto& [world, quests] : m_QuestPlayerList) {
            static_cast<void>(world);
            std::int32_t position = 1;
            for (QuestPlayerList& quest : quests) {
                if (quest.sendMessageTime <= queueNow) {
                    quest.sendMessageTime = queueNow + sendInterval;
                    SendQueuePosition(context, quest.socketId, position);
                    ++report.queuePositionMessages;
                }
                position = static_cast<std::int32_t>(
                    std::bit_cast<std::uint32_t>(position) + 1U);
            }
        }
        for (auto& [world, quests] : m_QuestPlayerData) {
            static_cast<void>(world);
            std::int32_t position = 1;
            for (QuestPlayerData& quest : quests) {
                if (quest.sendMessageTime <= queueNow) {
                    quest.sendMessageTime = queueNow + sendInterval;
                    SendQueuePosition(context, quest.socketId, position);
                    ++report.queuePositionMessages;
                }
                position = static_cast<std::int32_t>(
                    std::bit_cast<std::uint32_t>(position) + 1U);
            }
        }
    }

    ClearTimeoutList(context);
    report.authTimeoutsPublished =
        authManager.Run(context.AuthEventPublisher()).publishedTimeouts;

    std::uint32_t now = LegacyTickMs();
    if (m_MatricesLastTimeout + 1000U < now) {
        bool hasMatrices = false;
        {
            std::lock_guard guard(m_MatrixMutex);
            hasMatrices = !m_Matrices.empty();
        }
        if (hasMatrices) {
            m_MatricesLastTimeout = now;
            MatricesTimeout(context);
        }
    }

    now = LegacyTickMs();
    if (m_ValidCodeLastOvertime + 1000U < now) {
        bool hasValidCodes = false;
        {
            std::lock_guard guard(m_ValidCodeMutex);
            hasValidCodes = !m_ValidCodes.empty();
        }
        if (hasValidCodes) {
            m_ValidCodeLastOvertime = now;
            ValidCodeOvertime(context);
        }
    }

    now = LegacyTickMs();
    if (m_CheckValidErrInterval + 3000U < now) {
        m_CheckValidErrInterval = now;
        CheckValidErr(now);
    }

    return report;
}

NoQueueAccountsLoadResult CLoginQueue::LoadNoQueueCdkeyList(
    const std::filesystem::path& runtimeDirectory)
{
    {
        std::lock_guard guard(m_NoQueueMutex);
        m_NoQueueAccounts.clear();
    }

    std::error_code filesystemError;
    const auto path = ResolveWindowsAsset(runtimeDirectory, "NoQueueAccounts.conf", filesystemError);
    if (!path) {
        return NoQueueAccountsLoadResult{
            .error = NoQueueAccountsLoadError{
                .kind = NoQueueAccountsLoadErrorKind::Io,
                .detail = filesystemError ? filesystemError.message()
                                          : "NoQueueAccounts.conf не найден",
            },
        };
    }

    std::ifstream stream(*path, std::ios::binary);
    if (!stream.is_open()) {
        return NoQueueAccountsLoadResult{
            .error = NoQueueAccountsLoadError{
                .kind = NoQueueAccountsLoadErrorKind::Io,
                .detail = "NoQueueAccounts.conf не открыт",
            },
        };
    }

    const std::vector<std::uint8_t> bytes(
        std::istreambuf_iterator<char>(stream), std::istreambuf_iterator<char>{});
    std::size_t extracted = 0;
    std::size_t cursor = 0;
    while (cursor < bytes.size()) {
        while (cursor < bytes.size() && IsAsciiWhitespace(bytes[cursor])) {
            ++cursor;
        }
        if (cursor == bytes.size()) {
            break;
        }

        const std::size_t begin = cursor;
        while (cursor < bytes.size() && !IsAsciiWhitespace(bytes[cursor])) {
            ++cursor;
        }
        const std::size_t tokenIndex = extracted + 1U;
        const std::size_t tokenLength = cursor - begin;
        if (tokenLength >= kNoQueueAccountBufferSize) {
            std::lock_guard guard(m_NoQueueMutex);
            return NoQueueAccountsLoadResult{
                .extractedAccounts = extracted,
                .uniqueAccounts = m_NoQueueAccounts.size(),
                .error = NoQueueAccountsLoadError{
                    .kind = NoQueueAccountsLoadErrorKind::TokenTooLongLegacyOverflow,
                    .tokenIndex = tokenIndex,
                    .actualLength = tokenLength,
                },
            };
        }

        std::vector<std::uint8_t> account(bytes.begin() + static_cast<std::ptrdiff_t>(begin),
                                          bytes.begin() + static_cast<std::ptrdiff_t>(cursor));
        const auto nul = std::find(account.begin(), account.end(), std::uint8_t{0});
        account.erase(nul, account.end());
        if (std::any_of(account.begin(), account.end(),
                        [](std::uint8_t byte) { return byte >= 0x80U; })) {
            std::lock_guard guard(m_NoQueueMutex);
            return NoQueueAccountsLoadResult{
                .extractedAccounts = extracted,
                .uniqueAccounts = m_NoQueueAccounts.size(),
                .error = NoQueueAccountsLoadError{
                    .kind = NoQueueAccountsLoadErrorKind::NonAsciiCaseMappingUnknown,
                    .tokenIndex = tokenIndex,
                    .actualLength = account.size(),
                },
            };
        }

        LowerAscii(account);
        {
            std::lock_guard guard(m_NoQueueMutex);
            m_NoQueueAccounts.insert(std::move(account));
        }
        ++extracted;
    }

    std::lock_guard guard(m_NoQueueMutex);
    if (extracted == 0U) {
        return NoQueueAccountsLoadResult{
            .extractedAccounts = 0,
            .uniqueAccounts = m_NoQueueAccounts.size(),
            .error = NoQueueAccountsLoadError{
                .kind = NoQueueAccountsLoadErrorKind::EmptyFileLegacyReadUnknown,
            },
        };
    }
    return NoQueueAccountsLoadResult{
        .extractedAccounts = extracted,
        .uniqueAccounts = m_NoQueueAccounts.size(),
    };
}

bool CLoginQueue::IsInNoQueueList(std::span<const std::uint8_t> account) const
{
    const auto key = OwnedLegacyString(account);
    std::lock_guard guard(m_NoQueueMutex);
    return m_NoQueueAccounts.find(key) != m_NoQueueAccounts.end();
}

void CLoginQueue::AddQuestCdkey(std::int32_t socketId,
                                std::uint32_t clientIp,
                                std::int32_t loginType,
                                std::int32_t version,
                                std::span<const std::uint8_t> account,
                                std::span<const std::uint8_t> passwordDigest,
                                std::int16_t clientCode,
                                std::int32_t encryptionKey,
                                std::span<const std::uint8_t> worldServer)
{
    QuestCdkey quest{
        .socketId = socketId,
        .clientIp = clientIp,
        .loginType = static_cast<std::int8_t>(loginType),
        .version = version,
        .account = OwnedLegacyString(account),
        .passwordDigest = {passwordDigest.begin(), passwordDigest.end()},
        .clientCode = clientCode,
        .encryptionKey = encryptionKey,
        .worldServer = OwnedLegacyString(worldServer),
        .sendMessageTime = LegacyTickMs() + SendMessageIntervalMs(),
    };

    const bool noQueue = IsInNoQueueList(quest.account);
    std::lock_guard guard(m_QuestCdkeyMutex);
    if (noQueue) {
        m_NoQueueQuestCdkey.push_back(std::move(quest));
    } else {
        m_QuestCdkey.push_back(std::move(quest));
    }
}

void CLoginQueue::AddGasQueue(const QuestCdkey& quest)
{
    std::lock_guard guard(m_GasQueueMutex);
    m_GasQueue.push_back(quest);
}

std::optional<CLoginQueue::QuestCdkey> CLoginQueue::PopGasQueue()
{
    std::lock_guard guard(m_GasQueueMutex);
    if (m_GasQueue.empty()) {
        return std::nullopt;
    }
    QuestCdkey quest = std::move(m_GasQueue.front());
    m_GasQueue.pop_front();
    return quest;
}

void CLoginQueue::ClearGasQueue()
{
    std::lock_guard guard(m_GasQueueMutex);
    m_GasQueue.clear();
}

std::optional<QuestCdkeyError> CLoginQueue::OnQuestCdkey(
    ILoginQueueContext& context,
    AuthManager& authManager,
    const QuestCdkey& quest)
{
    if (!quest.worldServer.empty()) {
        TagPwdChecked checked(quest.socketId, quest.clientIp, quest.account,
                              quest.worldServer, false);
        switch (context.PrepareEnter(checked)) {
        case PrepareEnterOutcome::Continue:
            context.EnterGame(checked, IsInNoQueueList(checked.Account()));
            break;
        case PrepareEnterOutcome::Finished:
        case PrepareEnterOutcome::MatrixRegistrationRequired:
            // hasMatrix=false делает matrix-register недостижимым в точном
            // CGame::PrepareEnter; временный outcome не добавляет side effect.
            break;
        }
        return std::nullopt;
    }

    const auto insideMode = context.InsideUseMode();
    if (!insideMode) {
        return QuestCdkeyError{
            .kind = QuestCdkeyErrorKind::InsideModeMissing,
            .detail = "m_lIsInsideUse ещё не материализован",
        };
    }
    if (*insideMode != 1) {
        if (*insideMode == 0) {
            AddGasQueue(quest);
        }
        return std::nullopt;
    }

    if (!context.HasRsCdKeyOwner()) {
        return QuestCdkeyError{
            .kind = QuestCdkeyErrorKind::DatabaseOwnerMissing,
            .detail = "CRsCDKey owner для OnQuestCdkey отсутствует",
        };
    }

    std::vector<std::uint8_t> account = quest.account;
    // Исходный byte-loop считает пустую строку полностью цифровой и тоже
    // вызывает FixPtAcc; std::all_of сохраняет именно эту vacuous-ветку.
    if (IsAsciiNumeric(account)) {
        account = context.FixPtAccount(account);
    }

    const double banVariantTime = context.BanVariantTime(account);
    if (banVariantTime != 0.0) {
    // ПОДТВЕРЖДЕНО ДИЗАССЕМБЛИРОВАНИЕМ 0x0041A2A4..0x0041A35A: GetLocalTime
        // происходит до VariantTimeToSystemTime. Оставляем этот порядок даже
        // при вынесенном compatibility decoder.
        const auto now = CurrentLocalDateTime();
        if (!now) {
            return QuestCdkeyError{
                .kind = QuestCdkeyErrorKind::LocalTimeUnavailable,
                .detail = "системный localtime не вернул локальное время",
            };
        }
        const auto banTime = context.DecodeVariantTime(banVariantTime);
        if (!banTime) {
            return QuestCdkeyError{
                .kind = QuestCdkeyErrorKind::VariantTimeConversionFailed,
                .detail = "OLE DATE ban_time не преобразован в календарное время",
            };
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
            context.SendToClient(response, quest.socketId);
            return std::nullopt;
        }
    }

    if (!context.IpIsAllowed(quest.clientIp)) {
        SendLoginCode(context, quest.socketId, 0x12U);
        return std::nullopt;
    }
    if (context.IpIsForbidden(quest.clientIp)) {
        SendLoginCode(context, quest.socketId, 0x12U);
        return std::nullopt;
    }
    if (!context.IsBetweenIp(account, quest.clientIp)) {
        SendLoginCode(context, quest.socketId, 0x11U);
        return std::nullopt;
    }

    const bool hasMatrix = context.MatrixUsed(account);
    auto passwordHex = PasswordDigestHex(quest.passwordDigest);
    if (!passwordHex) {
        return QuestCdkeyError{
            .kind = QuestCdkeyErrorKind::PasswordDigestTooShort,
            .actualLength = quest.passwordDigest.size(),
            .detail = "password digest короче исходных 16 байт",
        };
    }

    if (context.IsConnectAS()) {
        LowerAscii(account);
        LowerAscii(*passwordHex);
        ClientSendQueue* sender = context.AuthSendQueue();
        IAuthListener* listener = context.AuthListener();
        if (sender == nullptr || listener == nullptr) {
            return QuestCdkeyError{
                .kind = QuestCdkeyErrorKind::AuthTransportMissing,
                .detail = "IsConnectAS=true без Auth send queue/listener",
            };
        }
        static_cast<void>(authManager.AddQuest(
            quest.clientIp, quest.socketId, account, *passwordHex, *sender, *listener));
        return std::nullopt;
    }

    const auto canonicalAccount = context.ValidateLocalPassword(account, *passwordHex);
    if (!canonicalAccount) {
        SendLoginCode(context, quest.socketId, 7U);
        return std::nullopt;
    }

    PushBackPwdChecked(
        TagPwdChecked(quest.socketId, quest.clientIp, *canonicalAccount,
                      quest.worldServer, hasMatrix),
        [&context](std::span<const std::uint8_t> duplicateAccount) {
            context.KickOut(duplicateAccount);
        });
    return std::nullopt;
}

bool CLoginQueue::AddQuestPlayerList(ILoginQueueContext& context,
                                     std::int32_t socketId,
                                     std::span<const std::uint8_t> account,
                                     std::span<const std::uint8_t> worldServer)
{
    const auto accountKey = OwnedLegacyString(account);
    const auto worldKey = OwnedLegacyString(worldServer);
    if (!context.IsExitWorld(worldKey) ||
        !context.LoginCdkeyWorldServer(accountKey).has_value() ||
        context.LoginWorldPlayerNumByName(worldKey) == -1) {
        return false;
    }

    QuestPlayerList quest{
        .socketId = socketId,
        .account = accountKey,
        .worldServer = worldKey,
        .sendMessageTime = LegacyTickMs() + SendMessageIntervalMs(),
    };
    const bool noQueue = IsInNoQueueList(accountKey);

    std::lock_guard guard(m_PlayerQuestMutex);
    auto& queues = noQueue ? m_NoQueueQuestPlayerList : m_QuestPlayerList;
    queues[worldKey].push_back(std::move(quest));
    return true;
}

bool CLoginQueue::AddQuestPlayerData(ILoginQueueContext& context,
                                     std::int32_t socketId,
                                     std::span<const std::uint8_t> account,
                                     std::int32_t playerId,
                                     std::uint32_t clientIp)
{
    const auto accountKey = OwnedLegacyString(account);
    const auto worldServer = context.LoginCdkeyWorldServer(accountKey);
    if (!worldServer) {
        return false;
    }

    QuestPlayerData quest{
        .socketId = socketId,
        .account = accountKey,
        .playerId = playerId,
        .clientIp = clientIp,
        .sendMessageTime = LegacyTickMs() + SendMessageIntervalMs(),
    };
    const bool noQueue = IsInNoQueueList(accountKey);

    std::lock_guard guard(m_PlayerQuestMutex);
    auto& queues = noQueue ? m_NoQueueQuestPlayerData : m_QuestPlayerData;
    queues[*worldServer].push_back(std::move(quest));
    return true;
}

void CLoginQueue::OnQuestPlayerData(ILoginQueueContext& context,
                                    const QuestPlayerData& quest)
{
    const auto worldServer = context.LoginCdkeyWorldServer(quest.account);
    if (IsValidQuest(context, quest.playerId)) {
        const auto world = worldServer
            ? std::optional<std::span<const std::uint8_t>>(std::span<const std::uint8_t>(*worldServer))
            : std::nullopt;
        context.L2WQuestDetailSend(world, quest.account, quest.playerId, quest.clientIp);

    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x0041B447..0x0041B463: отправка World — void;
        // PushLoginList вызывается сразу после него без проверки результата.
        static_cast<void>(PushLoginList(quest.playerId));
        return;
    }

    LoginNet::CMessage response(kPlayerDataRejectMessageType);
    response.Base().Add(static_cast<char>(0x1C));
    AddLegacyString(response, quest.account);
    context.SendToClientCdkey(response, quest.account);
}

bool CLoginQueue::IsValidQuest(ILoginQueueContext& context, std::int32_t playerId)
{
    std::lock_guard guard(m_LoginListMutex);
    const auto found = m_LoginList.find(playerId);
    if (found == m_LoginList.end()) {
        return true;
    }

    const std::uint32_t now = LegacyTickMs();
    const std::uint32_t deadline =
        found->second + context.QuestPlayerDataIntervalMs();
    if (now <= deadline) {
        return false;
    }

    m_LoginList.erase(found);
    return true;
}

bool CLoginQueue::PushLoginList(std::int32_t playerId)
{
    std::lock_guard guard(m_LoginListMutex);
    if (m_LoginList.find(playerId) != m_LoginList.end()) {
        return false;
    }
    m_LoginList.emplace(playerId, LegacyTickMs());
    return true;
}

void CLoginQueue::ClearTimeoutList(ILoginQueueContext& context)
{
    std::lock_guard guard(m_LoginListMutex);
    for (auto current = m_LoginList.begin(); current != m_LoginList.end();) {
        // VERIFIED_ASSEMBLY 0x00417350..0x00417364: GetGame/timeGetTime и
        // interval-read происходят на КАЖДОЙ итерации, а условие удаления
        // строгое: now > added + interval. Не hoist-им now за цикл.
        const std::uint32_t now = LegacyTickMs();
        const std::uint32_t deadline =
            current->second + context.QuestPlayerDataIntervalMs();
        if (now > deadline) {
            current = m_LoginList.erase(current);
        } else {
            ++current;
        }
    }
}

bool CLoginQueue::IsValidErrManyTimes(ILoginQueueContext& context,
                                         std::span<const std::uint8_t> account) const
{
    const auto accountKey = OwnedLegacyString(account);
    std::lock_guard guard(m_ValidErrMutex);
    const auto found = m_ValidErrors.find(accountKey);
    return found != m_ValidErrors.end() &&
           context.ValidErrorUpperLimit() <= found->second.errorTimes;
}

void CLoginQueue::MatricesTimeout(ILoginQueueContext& context)
{
    std::lock_guard guard(m_MatrixMutex);
    for (auto current = m_Matrices.begin(); current != m_Matrices.end();) {
        const std::uint32_t now = LegacyTickMs();
        if (context.MatrixTimeoutMs() < now - current->second.addedTime) {
            LoginNet::CMessage response(kLoginResponseMessageType);
            response.Base().Add(static_cast<char>('E'));
            context.SendToClient(response, current->second.socketId);
            current = m_Matrices.erase(current);
        } else {
            ++current;
        }
    }
}

void CLoginQueue::ValidCodeOvertime(ILoginQueueContext& context)
{
    std::lock_guard guard(m_ValidCodeMutex);
    for (auto current = m_ValidCodes.begin(); current != m_ValidCodes.end();) {
        const std::uint32_t now = LegacyTickMs();
        if (context.ValidCodeOvertimeMs() < now - current->second.addedTime) {
            LoginNet::CMessage response(kLoginResponseMessageType);
            response.Base().Add(static_cast<char>('M'));
            context.SendToClient(response, current->second.socketId);
            current = m_ValidCodes.erase(current);
        } else {
            ++current;
        }
    }
}

void CLoginQueue::CheckValidErr(std::uint32_t now)
{
    std::lock_guard guard(m_ValidErrMutex);
    for (auto current = m_ValidErrors.begin(); current != m_ValidErrors.end();) {
        if (current->second.nextLoginTime < now) {
            current = m_ValidErrors.erase(current);
        } else {
            ++current;
        }
    }
}

ClientLostCleanupReport CLoginQueue::OnClientLost(std::span<const std::uint8_t> account)
{
    const auto accountKey = OwnedLegacyString(account);
    ClientLostCleanupReport report;
    {
        std::lock_guard guard(m_QuestCdkeyMutex);
        report.cdkeyRemoved = EraseFirstAccount(m_QuestCdkey, accountKey);
    }
    {
        std::lock_guard guard(m_PlayerQuestMutex);
        report.playerListRemoved = EraseFirstAccountFromEachWorld(m_QuestPlayerList, accountKey);
        report.playerDataRemoved = EraseFirstAccountFromEachWorld(m_QuestPlayerData, accountKey);
    }
    return report;
}

void CLoginQueue::PushBackPwdChecked(TagPwdChecked checked,
                                     const KickOutCallback& kickOut)
{
    std::lock_guard guard(m_PwdCheckedMutex);
    const auto duplicate = std::find_if(
        m_PwdChecked.begin(), m_PwdChecked.end(), [&](const TagPwdChecked& pending) {
            return BytesEqual(pending.Account(), checked.Account());
        });
    if (duplicate != m_PwdChecked.end()) {
        if (kickOut) {
            kickOut(duplicate->Account());
        }
        m_PwdChecked.erase(duplicate);
    }
    m_PwdChecked.push_back(std::move(checked));
}

std::size_t CLoginQueue::PendingPwdChecked() const
{
    std::lock_guard guard(m_PwdCheckedMutex);
    return m_PwdChecked.size();
}

HandlePwdCheckedReport CLoginQueue::HandlePwdChecked(ILoginQueueContext& context)
{
    HandlePwdCheckedReport report;

    // VERIFIED_DISASSEMBLY 0x0041BB4A..0x0041C0CC: исходный
    // lockPwdChecked берётся один раз ДО проверки размера и освобождается
    // только после полного drain. Не сокращаем critical section до pop_front.
    std::lock_guard pwdGuard(m_PwdCheckedMutex);
    while (!m_PwdChecked.empty()) {
        TagPwdChecked checked = std::move(m_PwdChecked.front());
        m_PwdChecked.pop_front();
        ++report.processed;

        // pt_account в исходнике — std::string::c_str()-совместимый указатель и
        // для пустой строки не null. Поэтому проверяются только реальные
        // endpoint-поля; donor empty-account guard сюда не переносится.
        if (checked.SocketID() == 0 || checked.ClientIP() == 0U) {
            ++report.droppedInvalidEndpoint;
            continue;
        }

        if (IsValidErrManyTimes(context, checked.Account())) {
            LoginNet::CMessage response(kLoginResponseMessageType);
            response.Base().Add(static_cast<char>('Q'));
            context.SendToClient(response, checked.SocketID());
            ++report.rejectedByValidErrors;
            continue;
        }

        if (context.ValidCodeEnabled()) {
            const auto accountKey = OwnedLegacyString(checked.Account());

    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x0041BCB2..0x0041BD5D: map::find(account),
            // iterator != end -> N отправляется сохранённому socket без
            // сравнения с новым socket. Старый Linux donor добавлял сравнение,
            // которого в RU EXE нет.
            std::optional<std::int32_t> previousSocket;
            {
                std::lock_guard validCodeGuard(m_ValidCodeMutex);
                const auto found = m_ValidCodes.find(accountKey);
                if (found != m_ValidCodes.end()) {
                    previousSocket = found->second.socketId;
                }
            }
            if (previousSocket) {
                LoginNet::CMessage previous(kLoginResponseMessageType);
                previous.Base().Add(static_cast<char>('N'));
                context.SendToClient(previous, *previousSocket);
            }

            CValidCode validCode;
            if (auto error = CValidCode::Generate(std::filesystem::path{"."}, validCode)) {
                report.errors.push_back(PwdCheckedError{
                    .kind = PwdCheckedErrorKind::ValidCodeGeneration,
                    .account = accountKey,
                    .detail = error->detail,
                });
                continue;
            }

            const std::uint32_t now = LegacyTickMs();
            {
                std::lock_guard validCodeGuard(m_ValidCodeMutex);
                m_ValidCodes[accountKey] = ValidCodeEntry{
                    .socketId = checked.SocketID(),
                    .clientIp = checked.ClientIP(),
                    .addedTime = now,
                    .validCode = std::vector<std::uint8_t>(validCode.ValidCode().begin(),
                                                           validCode.ValidCode().end()),
                    .worldServer = OwnedLegacyString(checked.WorldServer()),
                    .hasMatrix = checked.HasMatrix(),
                    .changeTime = 0U,
                };
            }

            LoginNet::CMessage response(kLoginResponseMessageType);
            response.Base().Add(std::uint8_t{'J'});
            AddLegacyString(response, checked.Account());
            response.Base().Add(static_cast<std::int32_t>(kValidCodeBitmapLength));
            const auto bitmap = validCode.Bitmap();
            response.Base().Add(bitmap.data(), static_cast<std::int32_t>(bitmap.size()));
            context.SendToClient(response, checked.SocketID());
            ++report.generatedValidCodes;
            continue;
        }

        switch (context.PrepareEnter(checked)) {
        case PrepareEnterOutcome::Continue:
            context.EnterGame(checked, IsInNoQueueList(checked.Account()));
            ++report.enteredWithoutValidCode;
            break;
        case PrepareEnterOutcome::Finished:
            break;
        case PrepareEnterOutcome::MatrixRegistrationRequired:
            if (auto error = MatrixRegister(context, checked)) {
                report.errors.push_back(PwdCheckedError{
                    .kind = PwdCheckedErrorKind::MatrixRandom,
                    .account = OwnedLegacyString(checked.Account()),
                    .detail = error->detail,
                });
            }
            break;
        }
    }

    return report;
}

CheckMessageInfo CLoginQueue::CheckMsgInfo(std::span<const std::uint8_t> account,
                                           std::int32_t socketId) const
{
    const auto accountKey = OwnedLegacyString(account);
    std::lock_guard guard(m_ValidCodeMutex);
    const auto found = m_ValidCodes.find(accountKey);
    if (found == m_ValidCodes.end()) {
        return CheckMessageInfo::Missing;
    }
    if (found->second.socketId != socketId) {
        return CheckMessageInfo::SocketMismatch;
    }
    if (LegacyTickMs() < found->second.changeTime + 1000U) {
        return CheckMessageInfo::Frequent;
    }
    return CheckMessageInfo::Success;
}

ValidateValidCodeOutcome CLoginQueue::ValidateValidCode(
    std::int32_t socketId,
    std::uint32_t clientIp,
    std::span<const std::uint8_t> suppliedCode,
    std::span<const std::uint8_t> account)
{
    const auto accountKey = OwnedLegacyString(account);
    const auto code = OwnedLegacyString(suppliedCode);
    std::lock_guard guard(m_ValidCodeMutex);
    const auto found = m_ValidCodes.find(accountKey);
    if (found == m_ValidCodes.end()) {
        return {};
    }
    if (found->second.clientIp != clientIp || found->second.socketId != socketId) {
        m_ValidCodes.erase(found);
        return {};
    }
    if (found->second.validCode != code) {
        return {};
    }

    ValidateValidCodeOutcome outcome{
        .accepted = true,
        .worldServer = found->second.worldServer,
        .hasMatrix = found->second.hasMatrix,
    };
    m_ValidCodes.erase(found);
    return outcome;
}

void CLoginQueue::ChangeValidCode(std::span<const std::uint8_t> account,
                                  std::span<const std::uint8_t> validCode)
{
    const auto accountKey = OwnedLegacyString(account);
    std::lock_guard guard(m_ValidCodeMutex);
    const auto found = m_ValidCodes.find(accountKey);
    if (found == m_ValidCodes.end()) {
        return;
    }
    found->second.changeTime = LegacyTickMs();
    found->second.validCode = OwnedLegacyString(validCode);
}

void CLoginQueue::DelValidCode(std::span<const std::uint8_t> account)
{
    const auto accountKey = OwnedLegacyString(account);
    std::lock_guard guard(m_ValidCodeMutex);
    m_ValidCodes.erase(accountKey);
}

void CLoginQueue::AddValidErr(std::span<const std::uint8_t> account,
                              std::uint32_t stayTimeMs)
{
    const auto accountKey = OwnedLegacyString(account);
    const std::uint32_t nextLoginTime = LegacyTickMs() + stayTimeMs;
    std::lock_guard guard(m_ValidErrMutex);
    auto [found, inserted] = m_ValidErrors.try_emplace(accountKey);
    if (inserted) {
        found->second.errorTimes = 1;
    } else {
        const std::uint32_t wrapped =
            std::bit_cast<std::uint32_t>(found->second.errorTimes) + 1U;
        found->second.errorTimes = std::bit_cast<std::int32_t>(wrapped);
    }
    found->second.nextLoginTime = nextLoginTime;
}

MatrixValidationOutcome CLoginQueue::ValidateMatrix(
    ILoginQueueContext& context,
    std::int32_t socketId,
    std::uint32_t clientIp,
    std::span<const std::uint8_t> account,
    std::span<const std::uint8_t, 3> answer)
{
    const auto accountKey = OwnedLegacyString(account);
    std::lock_guard guard(m_MatrixMutex);
    const auto found = m_Matrices.find(accountKey);
    if (found == m_Matrices.end()) {
        return {};
    }
    if (found->second.clientIp != clientIp || found->second.socketId != socketId) {
        m_Matrices.erase(found);
        return {};
    }

    const auto validation = context.ValidateMatrixCard(
        accountKey,
        std::span<const std::uint8_t, 3>(found->second.positions),
        answer);
    if (validation.kind == MatrixCardValidationKind::OwnerMissing) {
        return MatrixValidationOutcome{
            .kind = MatrixValidationOutcomeKind::OwnerMissing,
        };
    }
    if (validation.kind == MatrixCardValidationKind::ValueTooShort) {
        return MatrixValidationOutcome{
            .kind = MatrixValidationOutcomeKind::ValueTooShort,
            .actualLength = validation.actualLength,
            .requiredLength = validation.requiredLength,
        };
    }

    const bool accepted = validation.accepted;
    m_Matrices.erase(found);
    return MatrixValidationOutcome{
        .kind = accepted ? MatrixValidationOutcomeKind::Accepted
                         : MatrixValidationOutcomeKind::Rejected,
    };
}

std::optional<MatrixRegisterError> CLoginQueue::MatrixRegister(
    ILoginQueueContext& context,
    const TagPwdChecked& checked)
{
    return MatrixRegister(
        checked,
        [&context](const LoginNet::CMessage& message, std::int32_t socketId) {
            context.SendToClient(message, socketId);
        });
}

std::optional<MatrixRegisterError> CLoginQueue::MatrixRegister(
    const TagPwdChecked& checked,
    const ClientSendCallback& sendToClient)
{
    if (checked.SocketID() == 0 || checked.ClientIP() == 0U) {
        return std::nullopt;
    }

    std::array<std::uint8_t, 3> positions{};
    for (auto& position : positions) {
        std::uint32_t random = 0;
        if (auto error = RandomWord(random)) {
            return error;
        }
        position = static_cast<std::uint8_t>(random % 0x50U);
    }

    std::optional<std::int32_t> previousSocket;
    {
        std::lock_guard guard(m_MatrixMutex);
        const auto found = m_Matrices.find(OwnedLegacyString(checked.Account()));
        if (found != m_Matrices.end()) {
            previousSocket = found->second.socketId;
        }
    }
    if (previousSocket) {
        LoginNet::CMessage previous(kLoginResponseMessageType);
        previous.Base().Add(static_cast<char>('F'));
        sendToClient(previous, *previousSocket);
        std::lock_guard guard(m_MatrixMutex);
        m_Matrices.erase(OwnedLegacyString(checked.Account()));
    }

    static_cast<void>(AddMatrix(checked.SocketID(),
                                checked.ClientIP(),
                                checked.Account(),
                                std::span<const std::uint8_t, 3>(positions)));

    LoginNet::CMessage response(kLoginResponseMessageType);
    response.Base().Add(static_cast<char>('B'));
    AddLegacyString(response, checked.Account());
    response.Base().Add(positions.data(), static_cast<std::int32_t>(positions.size()));
    sendToClient(response, checked.SocketID());
    return std::nullopt;
}

bool CLoginQueue::AddMatrix(std::int32_t socketId,
                            std::uint32_t clientIp,
                            std::span<const std::uint8_t> account,
                            std::span<const std::uint8_t, 3> positions)
{
    if (socketId == 0 || clientIp == 0U) {
        return false;
    }
    const auto accountKey = OwnedLegacyString(account);
    std::lock_guard guard(m_MatrixMutex);
    if (m_Matrices.find(accountKey) != m_Matrices.end()) {
        return false;
    }
    m_Matrices.emplace(accountKey,
                       MatrixEntry{
                           .socketId = socketId,
                           .clientIp = clientIp,
                           .positions = {positions[0], positions[1], positions[2]},
                           .addedTime = LegacyTickMs(),
                       });
    return true;
}

std::uint32_t CLoginQueue::SendMessageIntervalMs() const
{
    std::lock_guard guard(m_SetupMutex);
    return m_SendMessageIntervalMs;
}
}
