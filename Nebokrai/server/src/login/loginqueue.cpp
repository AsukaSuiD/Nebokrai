#include "loginqueue.h"

#include "authmanager.h"
#include "applogin/validcode.h"
#include "../nets/netlogin/message.h"

#include <algorithm>
#include <bit>
#include <cerrno>
#include <exception>
#include <fstream>
#include <iterator>
#include <random>
#include <string_view>
#include <system_error>
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
    m_WorldMaxPlayers = worldMaxPlayers;
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
    m_WorldCount = worldCount;
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
        std::istreambuf_iterator<char>(stream), std::istreambuf_iterator<char>());
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

        // VERIFIED_ASSEMBLY 0x0041B447..0x0041B463: World-send — void;
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

            // VERIFIED_ASSEMBLY 0x0041BCB2..0x0041BD5D: map::find(account),
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
        context.SendToClient(previous, *previousSocket);
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
    context.SendToClient(response, checked.SocketID());
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
