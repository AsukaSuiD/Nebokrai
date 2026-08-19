#include "loginqueue.h"

#include "../nets/netlogin/message.h"

#include <algorithm>
#include <bit>
#include <chrono>
#include <fstream>
#include <random>
#include <string>
#include <utility>

#if defined(__linux__)
#include <time.h>
#endif

namespace Login
{
namespace
{
constexpr std::size_t kNoQueueTokenBufferSize = 0x100U;
constexpr std::int32_t kAuthFailedMessageType = 0x000AF501;

std::vector<std::uint8_t> Key(std::span<const std::uint8_t> value)
{
    const auto end = std::find(value.begin(), value.end(), std::uint8_t{0});
    return {value.begin(), end};
}

bool Same(std::span<const std::uint8_t> left,
          std::span<const std::uint8_t> right)
{
    return left.size() == right.size() &&
           std::equal(left.begin(), left.end(), right.begin());
}

bool AsciiCaseEqual(std::string left, std::string right)
{
    const auto lower = [](unsigned char value) {
        if ('A' <= value && value <= 'Z') {
            return static_cast<char>(value - 'A' + 'a');
        }
        return static_cast<char>(value);
    };
    std::transform(left.begin(), left.end(), left.begin(), lower);
    std::transform(right.begin(), right.end(), right.begin(), lower);
    return left == right;
}

std::optional<std::filesystem::path>
ResolveNoQueueFile(const std::filesystem::path& runtimeDirectory)
{
    const auto exact = runtimeDirectory / "NoQueueAccounts.conf";
    std::error_code error;
    if (std::filesystem::is_regular_file(exact, error)) {
        return exact;
    }
    error.clear();
    for (const auto& entry : std::filesystem::directory_iterator(runtimeDirectory, error)) {
        if (error) {
            break;
        }
        if (AsciiCaseEqual(entry.path().filename().string(), "NoQueueAccounts.conf")) {
            return entry.path();
        }
    }
    return std::nullopt;
}

std::array<std::uint8_t, 3> RandomMatrixPositions()
{
    // В поздней Linux-реконструкции системный RNG заменял старый технический
    // источник случайности; доменная семантика здесь только modulo 0x50.
    std::random_device random;
    return {
        static_cast<std::uint8_t>(random() % 0x50U),
        static_cast<std::uint8_t>(random() % 0x50U),
        static_cast<std::uint8_t>(random() % 0x50U),
    };
}

void AddLegacyString(CBaseMessage& message, std::span<const std::uint8_t> value)
{
    const auto prefix = Key(value);
    if (!prefix.empty()) {
        message.Add(prefix.data(), static_cast<std::int32_t>(prefix.size()));
    }
    message.Add(std::uint8_t{0});
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

std::int32_t TagPwdChecked::SocketID() const noexcept { return m_SocketID; }
std::uint32_t TagPwdChecked::ClientIP() const noexcept { return m_ClientIP; }
std::span<const std::uint8_t> TagPwdChecked::Account() const noexcept { return m_Account; }
std::span<const std::uint8_t> TagPwdChecked::WorldServer() const noexcept { return m_WorldServer; }
bool TagPwdChecked::HasMatrix() const noexcept { return m_HasMatrix; }

CLoginQueue::CLoginQueue()
{
    static_cast<void>(LoadNoQueueCdkeyList(std::filesystem::current_path()));
}

void CLoginQueue::OnInitial(std::uint32_t intervalMs,
                            std::uint32_t sendMessageIntervalMs,
                            std::uint32_t worldMaxPlayers)
{
    std::lock_guard<std::mutex> lock(m_SetupMutex);
    m_Setup.intervalMs = intervalMs;
    m_Setup.sendMessageIntervalMs = sendMessageIntervalMs;
    m_Setup.worldMaxPlayers = worldMaxPlayers;
    if (m_Setup.worldCount == 0U) {
        m_Setup.worldCount = 1U;
    }
    const std::uint32_t now = LegacyTickMs();
    m_Setup.worldQueueTime = now + intervalMs;
    m_Setup.logQueueTime = now + intervalMs / m_Setup.worldCount;
}

NoQueueLoadResult
CLoginQueue::LoadNoQueueCdkeyList(const std::filesystem::path& runtimeDirectory)
{
    std::lock_guard<std::mutex> lock(m_NoQueueMutex);
    m_NoQueueAccounts.clear();

    const auto path = ResolveNoQueueFile(runtimeDirectory);
    if (!path) {
        return {.status = NoQueueLoadStatus::IoError};
    }

    std::ifstream input(*path, std::ios::binary);
    if (!input) {
        return {.status = NoQueueLoadStatus::IoError};
    }

    std::size_t extracted = 0U;
    std::string token;
    while (input >> token) {
        const std::size_t tokenIndex = extracted;
        if (token.size() >= kNoQueueTokenBufferSize) {
            return {
                .status = NoQueueLoadStatus::TokenTooLong,
                .extractedAccounts = extracted,
                .uniqueAccounts = m_NoQueueAccounts.size(),
                .tokenIndex = tokenIndex,
                .tokenLength = token.size(),
            };
        }
        if (std::any_of(token.begin(), token.end(), [](unsigned char byte) {
                return byte >= 0x80U;
            })) {
            return {
                .status = NoQueueLoadStatus::NonAsciiCaseMappingUnknown,
                .extractedAccounts = extracted,
                .uniqueAccounts = m_NoQueueAccounts.size(),
                .tokenIndex = tokenIndex,
                .tokenLength = token.size(),
            };
        }

        std::vector<std::uint8_t> account(token.begin(), token.end());
        for (auto& byte : account) {
            if ('A' <= byte && byte <= 'Z') {
                byte = static_cast<std::uint8_t>(byte - 'A' + 'a');
            }
        }
        m_NoQueueAccounts.insert(std::move(account));
        ++extracted;
    }

    if (extracted == 0U) {
        return {
            .status = NoQueueLoadStatus::EmptyFileLegacyReadUndefined,
            .uniqueAccounts = m_NoQueueAccounts.size(),
        };
    }
    return {
        .status = NoQueueLoadStatus::Loaded,
        .extractedAccounts = extracted,
        .uniqueAccounts = m_NoQueueAccounts.size(),
    };
}

bool CLoginQueue::IsInNoQueueList(std::span<const std::uint8_t> account) const
{
    const auto key = Key(account);
    std::lock_guard<std::mutex> lock(m_NoQueueMutex);
    return m_NoQueueAccounts.contains(key);
}

void CLoginQueue::AddQuestCdkey(std::int32_t socketId,
                                std::uint32_t clientIp,
                                std::int32_t loginType,
                                std::int32_t version,
                                std::vector<std::uint8_t> account,
                                std::vector<std::uint8_t> passwordDigest,
                                std::int16_t clientCode,
                                std::int32_t encryptionKey,
                                std::vector<std::uint8_t> worldServer)
{
    std::uint32_t sendInterval = 0U;
    {
        std::lock_guard<std::mutex> lock(m_SetupMutex);
        sendInterval = m_Setup.sendMessageIntervalMs;
    }
    const std::uint32_t sendTime = LegacyTickMs() + sendInterval;
    const bool noQueue = IsInNoQueueList(account);

    QuestCdkey quest{
        .socketId = socketId,
        .clientIp = clientIp,
        .loginType = static_cast<std::int8_t>(loginType),
        .version = version,
        .account = std::move(account),
        .passwordDigest = std::move(passwordDigest),
        .clientCode = clientCode,
        .encryptionKey = encryptionKey,
        .worldServer = std::move(worldServer),
        .sendMessageTime = sendTime,
    };

    std::lock_guard<std::mutex> lock(m_CdkeyQuestMutex);
    if (noQueue) {
        m_NoQueueQuestCdkey.push_back(std::move(quest));
    } else {
        m_QuestCdkey.push_back(std::move(quest));
    }
}

bool CLoginQueue::AddQuestPlayerList(ILoginQueueContext& context,
                                     std::int32_t socketId,
                                     std::span<const std::uint8_t> account,
                                     std::span<const std::uint8_t> worldServer)
{
    if (!context.IsExitWorld(worldServer) ||
        !context.LoginCdkeyWorldServer(account).has_value() ||
        context.LoginWorldPlayerNumByName(worldServer) == -1) {
        return false;
    }

    std::uint32_t sendInterval = 0U;
    {
        std::lock_guard<std::mutex> lock(m_SetupMutex);
        sendInterval = m_Setup.sendMessageIntervalMs;
    }
    QuestPlayerList quest{
        .socketId = socketId,
        .account = Key(account),
        .worldServer = Key(worldServer),
        .sendMessageTime = LegacyTickMs() + sendInterval,
    };
    const auto worldKey = quest.worldServer;
    const bool noQueue = IsInNoQueueList(account);

    std::lock_guard<std::mutex> lock(m_PlayerListQuestMutex);
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
    const auto worldServer = context.LoginCdkeyWorldServer(account);
    if (!worldServer) {
        return false;
    }

    std::uint32_t sendInterval = 0U;
    {
        std::lock_guard<std::mutex> lock(m_SetupMutex);
        sendInterval = m_Setup.sendMessageIntervalMs;
    }
    QuestPlayerData quest{
        .socketId = socketId,
        .account = Key(account),
        .playerId = playerId,
        .clientIp = clientIp,
        .sendMessageTime = LegacyTickMs() + sendInterval,
    };
    const bool noQueue = IsInNoQueueList(account);

    std::lock_guard<std::mutex> lock(m_PlayerDataQuestMutex);
    auto& queues = noQueue ? m_NoQueueQuestPlayerData : m_QuestPlayerData;
    queues[Key(*worldServer)].push_back(std::move(quest));
    return true;
}

ClientLostCleanupReport
CLoginQueue::OnClientLost(std::span<const std::uint8_t> account)
{
    const auto key = Key(account);
    ClientLostCleanupReport report;

    {
        std::lock_guard<std::mutex> lock(m_CdkeyQuestMutex);
        const auto found = std::find_if(m_QuestCdkey.begin(), m_QuestCdkey.end(),
                                        [&](const QuestCdkey& quest) {
                                            return quest.account == key;
                                        });
        if (found != m_QuestCdkey.end()) {
            m_QuestCdkey.erase(found);
            report.cdkeyRemoved = true;
        }
    }

    {
        std::lock_guard<std::mutex> lock(m_PlayerListQuestMutex);
        for (auto& [_, queue] : m_QuestPlayerList) {
            const auto found = std::find_if(queue.begin(), queue.end(),
                                            [&](const QuestPlayerList& quest) {
                                                return quest.account == key;
                                            });
            if (found != queue.end()) {
                queue.erase(found);
                ++report.playerListRemoved;
            }
        }
    }

    {
        std::lock_guard<std::mutex> lock(m_PlayerDataQuestMutex);
        for (auto& [_, queue] : m_QuestPlayerData) {
            const auto found = std::find_if(queue.begin(), queue.end(),
                                            [&](const QuestPlayerData& quest) {
                                                return quest.account == key;
                                            });
            if (found != queue.end()) {
                queue.erase(found);
                ++report.playerDataRemoved;
            }
        }
    }

    return report;
}

void CLoginQueue::PushBackPwdChecked(TagPwdChecked checked,
                                     const KickOutCallback& kickOut)
{
    std::lock_guard<std::mutex> lock(m_PwdCheckedMutex);
    const auto found = std::find_if(m_PwdChecked.begin(), m_PwdChecked.end(),
                                    [&](const TagPwdChecked& pending) {
                                        return Same(pending.Account(), checked.Account());
                                    });
    if (found != m_PwdChecked.end()) {
        if (kickOut) {
            kickOut(found->Account());
        }
        m_PwdChecked.erase(found);
    }
    m_PwdChecked.push_back(std::move(checked));
}

std::size_t CLoginQueue::PendingPwdChecked() const
{
    std::lock_guard<std::mutex> lock(m_PwdCheckedMutex);
    return m_PwdChecked.size();
}

CheckMessageInfo
CLoginQueue::CheckMsgInfo(std::span<const std::uint8_t> account,
                          std::int32_t socketId) const
{
    const auto key = Key(account);
    std::lock_guard<std::mutex> lock(m_ValidCodeMutex);
    const auto found = m_ValidCodes.find(key);
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

void CLoginQueue::PutValidCode(std::int32_t socketId,
                               std::uint32_t clientIp,
                               std::span<const std::uint8_t> account,
                               std::span<const std::uint8_t> validCode,
                               std::span<const std::uint8_t> worldServer,
                               bool hasMatrix)
{
    const std::uint32_t now = LegacyTickMs();
    std::lock_guard<std::mutex> lock(m_ValidCodeMutex);
    m_ValidCodes[Key(account)] = ValidCodeEntry{
        .socketId = socketId,
        .clientIp = clientIp,
        .addedTime = now,
        .validCode = Key(validCode),
        .worldServer = Key(worldServer),
        .hasMatrix = hasMatrix,
        .changeTime = 0U,
    };
}

ValidateValidCodeResult
CLoginQueue::ValidateValidCode(std::int32_t socketId,
                               std::uint32_t clientIp,
                               std::span<const std::uint8_t> suppliedCode,
                               std::span<const std::uint8_t> account)
{
    const auto key = Key(account);
    const auto supplied = Key(suppliedCode);
    std::lock_guard<std::mutex> lock(m_ValidCodeMutex);
    const auto found = m_ValidCodes.find(key);
    if (found == m_ValidCodes.end()) {
        return {};
    }
    if (found->second.clientIp != clientIp || found->second.socketId != socketId) {
        m_ValidCodes.erase(found);
        return {};
    }
    if (found->second.validCode != supplied) {
        return {};
    }

    ValidateValidCodeResult result{
        .accepted = true,
        .worldServer = found->second.worldServer,
        .hasMatrix = found->second.hasMatrix,
    };
    m_ValidCodes.erase(found);
    return result;
}

void CLoginQueue::ChangeValidCode(std::span<const std::uint8_t> account,
                                  std::span<const std::uint8_t> validCode)
{
    const auto key = Key(account);
    std::lock_guard<std::mutex> lock(m_ValidCodeMutex);
    const auto found = m_ValidCodes.find(key);
    if (found == m_ValidCodes.end()) {
        return;
    }
    found->second.changeTime = LegacyTickMs();
    found->second.validCode = Key(validCode);
}

void CLoginQueue::DelValidCode(std::span<const std::uint8_t> account)
{
    std::lock_guard<std::mutex> lock(m_ValidCodeMutex);
    m_ValidCodes.erase(Key(account));
}

void CLoginQueue::AddValidErr(std::span<const std::uint8_t> account,
                              std::uint32_t stayTimeMs)
{
    const std::uint32_t nextLoginTime = LegacyTickMs() + stayTimeMs;
    const auto key = Key(account);
    std::lock_guard<std::mutex> lock(m_ValidErrMutex);
    const auto found = m_ValidErrors.find(key);
    if (found == m_ValidErrors.end()) {
        m_ValidErrors.emplace(key, ValidErrEntry{1, nextLoginTime});
        return;
    }
    const auto wrapped = std::bit_cast<std::uint32_t>(found->second.errorTimes) + 1U;
    found->second.errorTimes = std::bit_cast<std::int32_t>(wrapped);
    found->second.nextLoginTime = nextLoginTime;
}

bool CLoginQueue::AddMatrix(std::int32_t socketId,
                            std::uint32_t clientIp,
                            std::span<const std::uint8_t> account,
                            std::span<const std::uint8_t, 3> positions)
{
    if (socketId == 0 || clientIp == 0U) {
        return false;
    }
    const auto key = Key(account);
    std::lock_guard<std::mutex> lock(m_MatrixMutex);
    if (m_Matrices.contains(key)) {
        return false;
    }
    m_Matrices.emplace(key, MatrixEntry{
        .socketId = socketId,
        .clientIp = clientIp,
        .positions = {positions[0], positions[1], positions[2]},
        .addedTime = LegacyTickMs(),
    });
    return true;
}

MatrixValidationOutcome
CLoginQueue::ValidateMatrix(ILoginQueueContext& context,
                            std::int32_t socketId,
                            std::uint32_t clientIp,
                            std::span<const std::uint8_t> account,
                            std::span<const std::uint8_t, 3> answer)
{
    const auto key = Key(account);
    std::lock_guard<std::mutex> lock(m_MatrixMutex);
    const auto found = m_Matrices.find(key);
    if (found == m_Matrices.end()) {
        return MatrixValidationOutcome::Rejected;
    }
    if (found->second.clientIp != clientIp || found->second.socketId != socketId) {
        m_Matrices.erase(found);
        return MatrixValidationOutcome::Rejected;
    }

    const auto positions = std::span<const std::uint8_t, 3>(found->second.positions);
    const MatrixCardValidation validation =
        context.ValidateMatrixCard(account, positions, answer);
    if (validation.kind == MatrixCardValidationKind::DatabaseOwnerMissing) {
        return MatrixValidationOutcome::DatabaseOwnerMissing;
    }
    if (validation.kind == MatrixCardValidationKind::ValueTooShort) {
        return MatrixValidationOutcome::DatabaseValueTooShort;
    }

    const bool accepted = validation.accepted;
    m_Matrices.erase(found);
    return accepted ? MatrixValidationOutcome::Accepted
                    : MatrixValidationOutcome::Rejected;
}

void CLoginQueue::ContinueValidatedLogin(ILoginQueueContext& context,
                                         const TagPwdChecked& checked)
{
    const bool noQueue = IsInNoQueueList(checked.Account());
    switch (context.PrepareEnter(checked)) {
    case PrepareEnterOutcome::Continue:
        context.EnterGame(checked, noQueue);
        break;
    case PrepareEnterOutcome::Finished:
        break;
    case PrepareEnterOutcome::MatrixRegistrationRequired:
        MatrixRegister(context, checked);
        break;
    }
}

void CLoginQueue::MatrixRegister(ILoginQueueContext& context,
                                 const TagPwdChecked& checked)
{
    if (checked.SocketID() == 0 || checked.ClientIP() == 0U) {
        return;
    }
    const auto positions = RandomMatrixPositions();
    const auto key = Key(checked.Account());

    std::optional<std::int32_t> previousSocket;
    {
        std::lock_guard<std::mutex> lock(m_MatrixMutex);
        const auto found = m_Matrices.find(key);
        if (found != m_Matrices.end()) {
            previousSocket = found->second.socketId;
        }
    }
    if (previousSocket) {
        LoginNet::CMessage previous(kAuthFailedMessageType);
        previous.Base().Add('F');
        context.SendToClient(previous, *previousSocket);
        std::lock_guard<std::mutex> lock(m_MatrixMutex);
        m_Matrices.erase(key);
    }

    static_cast<void>(AddMatrix(checked.SocketID(), checked.ClientIP(),
                                checked.Account(), positions));

    LoginNet::CMessage response(kAuthFailedMessageType);
    response.Base().Add('B');
    AddLegacyString(response.Base(), checked.Account());
    response.Base().AddEx(positions.data(), static_cast<std::int32_t>(positions.size()));
    context.SendToClient(response, checked.SocketID());
}

std::uint32_t CLoginQueue::LegacyTickMs() noexcept
{
#if defined(__linux__) && defined(CLOCK_BOOTTIME)
    timespec now{};
    if (::clock_gettime(CLOCK_BOOTTIME, &now) == 0) {
        const std::uint64_t milliseconds =
            static_cast<std::uint64_t>(now.tv_sec) * 1000ULL +
            static_cast<std::uint64_t>(now.tv_nsec) / 1'000'000ULL;
        return static_cast<std::uint32_t>(milliseconds);
    }
#endif
    const auto now = std::chrono::steady_clock::now().time_since_epoch();
    return static_cast<std::uint32_t>(
        std::chrono::duration_cast<std::chrono::milliseconds>(now).count());
}
}
