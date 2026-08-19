#include "cgame.h"

#include "../nets/mysocket.h"

#include <algorithm>
#include <array>
#include <bit>
#include <ctime>
#include <exception>
#include <mutex>
#include <span>
#include <thread>
#include <type_traits>
#include <utility>

#if defined(__linux__)
#include <time.h>
#endif

namespace
{
constexpr auto kAcceptThreadDelay = std::chrono::milliseconds(100);
constexpr auto kAcceptAtCapacityDelay = std::chrono::seconds(1);
constexpr auto kMainLoopDelay = std::chrono::milliseconds(1);

struct ServerInfoTimers
{
    std::optional<std::uint32_t> lastUpdateMs;
    std::optional<std::uint32_t> lastWriteMs;
};

std::mutex g_ServerInfoTimersMutex;
ServerInfoTimers g_ServerInfoTimers;

void AddLegacyString(CBaseMessage& message, std::span<const std::uint8_t> value)
{
    const auto end = std::find(value.begin(), value.end(), std::uint8_t{0});
    const std::size_t length = static_cast<std::size_t>(std::distance(value.begin(), end));
    if (length != 0U) {
        message.Add(value.data(), static_cast<std::int32_t>(length));
    }
    message.Add(std::uint8_t{0});
}

AuthProcessStatus SendAuthResultMessage(const ServerCommandHandle& sender,
                                        const AuthDb::AuthenticateResult& value)
{
    CMessage message(0x000CF601);
    message.Base().Add(value.result.result);
    AddLegacyString(message.Base(), value.result.account);
    message.Base().Add(value.result.clientIp);
    message.Base().Add(value.result.clientSocketId);
    return std::holds_alternative<AuthSendMessageError>(
               message.SendToLogin(sender, value.returnSocketId))
               ? AuthProcessStatus::MessageBuildFailed
               : AuthProcessStatus::Ok;
}

AuthProcessStatus SendAuthExtendedResultMessage(
    const ServerCommandHandle& sender,
    const AuthDb::AuthenticateExtendedResult& value)
{
    CMessage message(0x000CF602);
    message.Base().Add(value.result.result);
    AddLegacyString(message.Base(), value.result.account);
    message.Base().Add(value.result.clientIp);
    message.Base().Add(value.result.clientSocketId);

    if (value.result.result == 3) {
        // RVA 0x4460 отправляет year/month/day/hour/minute; day-of-week и
        // seconds из SYSTEMTIME не попадают в wire.
        for (const std::size_t offset : {0U, 2U, 6U, 8U, 10U}) {
            const std::uint16_t word =
                static_cast<std::uint16_t>(value.result.extra[offset]) |
                (static_cast<std::uint16_t>(value.result.extra[offset + 1]) << 8U);
            message.Base().Add(word);
        }
    } else if (value.result.result == 7) {
        message.Base().Add(value.result.extra.data(),
                           static_cast<std::int32_t>(value.result.extra.size()));
    }

    return std::holds_alternative<AuthSendMessageError>(
               message.SendToLogin(sender, value.returnSocketId))
               ? AuthProcessStatus::MessageBuildFailed
               : AuthProcessStatus::Ok;
}

AuthProcessStatus SendLockResultMessage(const ServerCommandHandle& sender,
                                        const AuthDb::LockResult& value)
{
    CMessage message(0x0010F202);
    AddLegacyString(message.Base(), value.result.account);
    message.Base().Add(static_cast<std::uint8_t>(value.result.succeeded ? 1U : 0U));
    return std::holds_alternative<AuthSendMessageError>(
               message.SendToLogin(sender, value.returnSocketId))
               ? AuthProcessStatus::MessageBuildFailed
               : AuthProcessStatus::Ok;
}
}

struct AuthDbContext::State
{
    explicit State(ConfigReader value)
        : config(std::move(value))
    {
    }

    mutable std::shared_mutex configMutex;
    ConfigReader config;
    MultiList<AuthDb::DbQuest> quests;
    MultiList<AuthDb::DbResult> results;
    AuthDb::ServerInfoQueue serverInfo;
    mutable std::shared_mutex clientIpMutex;
    kl_net::IpFilter<false> clientIpForbider;
};

AuthDbContext::AuthDbContext(ConfigReader config)
    : m_State(std::make_shared<State>(std::move(config)))
{
}

ConfigReader AuthDbContext::ConfigSnapshot() const
{
    std::shared_lock<std::shared_mutex> lock(m_State->configMutex);
    return m_State->config;
}

ConfigLoadResult AuthDbContext::LoadConfig(const std::filesystem::path& path)
{
    std::unique_lock<std::shared_mutex> lock(m_State->configMutex);
    return m_State->config.Load(path);
}

void AuthDbContext::ResetConfig()
{
    std::unique_lock<std::shared_mutex> lock(m_State->configMutex);
    m_State->config.Reset();
}

bool AuthDbContext::PushQuest(AuthDb::DbQuest quest)
{
    const ConfigReader config = ConfigSnapshot();
    if (m_State->quests.Size() < static_cast<std::uint32_t>(config.MaxAuthQueueSize())) {
        m_State->quests.PushBack(std::move(quest));
        return true;
    }

    std::visit(
        [this](auto&& value) {
            using T = std::decay_t<decltype(value)>;
            if constexpr (std::is_same_v<T, AuthDb::AuthenticateQuest>) {
                PushResult(AuthDb::AuthenticateResult{
                    value.returnSocketId,
                    AuthDb::AuthResultData{
                        6,
                        std::move(value.request.account),
                        value.request.clientIp,
                        value.request.clientSocketId}});
            } else if constexpr (std::is_same_v<T, AuthDb::AuthenticateExtendedQuest>) {
                PushResult(AuthDb::AuthenticateExtendedResult{
                    value.returnSocketId,
                    AuthDb::AuthExResultData{
                        6,
                        std::move(value.request.account),
                        value.request.clientIp,
                        value.request.clientSocketId,
                        {}}});
            } else if constexpr (std::is_same_v<T, AuthDb::LockQuest>) {
                PushResult(AuthDb::LockResult{
                    value.returnSocketId,
                    AuthDb::LockResultData{std::move(value.request.account), false}});
            }
            // WriteServerInfo при переполнении результата не создаёт.
        },
        std::move(quest));
    return false;
}

std::uint32_t AuthDbContext::QuestCount() const
{
    return m_State->quests.Size();
}

std::optional<AuthDb::DbQuest>
AuthDbContext::PopQuestUntilStopped(const std::atomic_bool& stopped)
{
    return m_State->quests.PopFrontWaitUntilStopped(stopped);
}

void AuthDbContext::WakeQuestWaiters()
{
    m_State->quests.WakeAll();
}

void AuthDbContext::PushResult(AuthDb::DbResult result)
{
    m_State->results.PushBack(std::move(result));
}

std::uint32_t AuthDbContext::ResultCount() const
{
    return m_State->results.Size();
}

AuthDb::DbResult AuthDbContext::PopResultWait()
{
    return m_State->results.PopFrontWait();
}

void AuthDbContext::PushServerInfo(AuthDb::ServerInfo info)
{
    m_State->serverInfo.PushBack(std::move(info));
}

std::deque<AuthDb::ServerInfo> AuthDbContext::PopAllServerInfo()
{
    return m_State->serverInfo.PopAll();
}

void AuthDbContext::ReplaceClientForbidPatterns(std::vector<kl_net::IpPattern> patterns)
{
    std::unique_lock<std::shared_mutex> lock(m_State->clientIpMutex);
    m_State->clientIpForbider.ReplacePatterns(std::move(patterns));
}

bool AuthDbContext::IsClientIpAllowed(std::uint32_t address) const
{
    if (!ConfigSnapshot().ClientIpFilterEnabled()) {
        return true;
    }
    const kl_net::IpPattern octets{
        static_cast<std::uint8_t>(address),
        static_cast<std::uint8_t>(address >> 8U),
        static_cast<std::uint8_t>(address >> 16U),
        static_cast<std::uint8_t>(address >> 24U),
    };
    std::shared_lock<std::shared_mutex> lock(m_State->clientIpMutex);
    return m_State->clientIpForbider.IsAllowed(octets);
}

CGame::CGame(ConfigReader config,
             AuthDatabaseFactory databaseFactory,
             std::vector<kl_net::IpPattern> loginServerPatterns)
    : m_Db(std::move(config)),
      m_DatabaseFactory(std::move(databaseFactory)),
      m_MessageHandlers(
          false,
          {},
          [context = m_Db](AuthDb::DbQuest quest) mutable {
              return context.PushQuest(std::move(quest));
          },
          [context = m_Db](AuthDb::ServerInfo info) mutable {
              context.PushServerInfo(std::move(info));
          })
{
    const ConfigReader snapshot = m_Db.ConfigSnapshot();
    if (!snapshot.LoginServerIpFilterEnabled()) {
        loginServerPatterns.clear();
    }
    m_MessageHandlers.ReplaceIpFilter(snapshot.LoginServerIpFilterEnabled(),
                                      std::move(loginServerPatterns));
    m_NextAcceptAt = std::chrono::steady_clock::now();
}

CGame::~CGame()
{
    static_cast<void>(ReleaseRuntime());
}

AuthInitializationResult CGame::InitializeRuntime(
    const std::filesystem::path& setupPath,
    const std::filesystem::path& allowedClientsPath,
    const std::filesystem::path& clientForbidPath)
{
    AuthInitializationResult report;

    const ConfigLoadResult loadResult = m_Db.LoadConfig(setupPath);
    if (!loadResult) {
        m_Db.ResetConfig();
        report.notices.push_back({
            AuthInitializationNoticeType::SetupDefaultsApplied,
            loadResult,
            {},
        });
    }

    const ConfigReader config = m_Db.ConfigSnapshot();
    m_MessageHandlers.ReplaceIpFilter(config.LoginServerIpFilterEnabled(), {});

    bool allowedClientsLoaded = false;
    if (!InitializeNetwork(config.NetworkConfig(),
                           allowedClientsPath,
                           allowedClientsLoaded,
                           report.networkError)) {
        report.status = AuthInitializationStatus::NetworkFailed;
        return report;
    }
    if (!allowedClientsLoaded) {
        report.notices.push_back({
            AuthInitializationNoticeType::AllowedClientsUnavailable,
            {},
            {},
        });
    }

    report.databaseWorkers = m_DbWorkers.Start(
        m_Db,
        config.DatabaseThreadCount(),
        m_DatabaseFactory);
    if (!report.databaseWorkers) {
        report.status = AuthInitializationStatus::DatabaseWorkersFailed;
        return report;
    }

    m_Db.ReplaceClientForbidPatterns({});
    if (config.ClientIpFilterEnabled()) {
        kl_net::IpFilterLoadResult filter = kl_net::LoadIpPatterns(clientForbidPath);
        if (filter) {
            m_Db.ReplaceClientForbidPatterns(std::move(filter.patterns));
        } else {
            report.notices.push_back({
                AuthInitializationNoticeType::ClientIpFilterUnavailable,
                {},
                std::move(filter),
            });
        }
    }

    report.status = AuthInitializationStatus::Ok;
    return report;
}

AuthRuntimeStep CGame::RunNetworkTurn()
{
    AuthRuntimeStep step;
    if (!m_NetServerAuth) {
        step.status = AuthProcessStatus::NetworkNotInitialized;
        return step;
    }

    PumpIo();
    const std::uint32_t nowMs = AuthLegacyTickMs();
    HandleAcceptCompletion(nowMs, step);
    StartAcceptIfDue();

    ServerSnapshot snapshot = m_NetServerAuth->ProcessCommandSnapshot(nowMs);
    step.processedNetworkCommands = snapshot.processedCommands;
    step.networkErrors = std::move(snapshot.errors);
    SpawnIoActions(std::move(snapshot.ioActions), m_NetServerAuth->CommandHandle());

    step.status = ProcessMessages(step.processedMessages);
    step.networkTaskFailures = m_NetworkTaskFailures;
    return step;
}

AuthRuntimeStep CGame::RunMainLoopTurn()
{
    AuthRuntimeStep step = RunNetworkTurn();
    if (step.status != AuthProcessStatus::Ok) {
        return step;
    }

    step.status = ProcessDatabaseResults(step.processedDatabaseResults);
    if (step.status != AuthProcessStatus::Ok) {
        return step;
    }

    if (m_Db.ConfigSnapshot().UpdateServerInfoEnabled()) {
        const ServerInfoUpdate update = UpdateServerInfo();
        step.status = update.status;
        step.serverInfoRequested = update.requested;
        step.serverInfoWriteDue = update.writeDue;
        step.serverInfoWriteQueued = update.writeQueued;
    }

    std::this_thread::sleep_for(kMainLoopDelay);
    return step;
}

AuthReleaseResult CGame::ReleaseRuntime()
{
    AuthReleaseResult result;
    result.databaseWorkerErrors = m_DbWorkers.StopAndJoin();
    ReleaseNetwork();
    result.networkTaskFailures = m_NetworkTaskFailures;
    return result;
}

AuthProcessStatus CGame::ProcessMessages(std::int32_t& processed)
{
    processed = 0;
    if (!m_NetServerAuth) {
        return AuthProcessStatus::NetworkNotInitialized;
    }

    const ServerCommandHandle sender = m_NetServerAuth->CommandHandle();
    std::int32_t remaining = m_NetServerAuth->PendingMessages();
    while (remaining > 0) {
        if (std::unique_ptr<CMessage> message = m_NetServerAuth->PopReceivedMessage()) {
            const AuthDispatchStatus status = message->Run(m_MessageHandlers, sender);
            if (status != AuthDispatchStatus::Ok) {
                return status == AuthDispatchStatus::OutgoingMessageTooLarge
                           ? AuthProcessStatus::MessageBuildFailed
                           : AuthProcessStatus::DispatchFailed;
            }
            processed = std::bit_cast<std::int32_t>(
                std::bit_cast<std::uint32_t>(processed) + std::uint32_t{1});
        }
        --remaining;
    }
    return AuthProcessStatus::Ok;
}

AuthProcessStatus CGame::ProcessDatabaseResults(std::int32_t& processed)
{
    processed = 0;
    if (!m_NetServerAuth) {
        return AuthProcessStatus::NetworkNotInitialized;
    }

    const ServerCommandHandle sender = m_NetServerAuth->CommandHandle();
    std::uint32_t remaining = m_Db.ResultCount();
    while (remaining > 0U) {
        AuthDb::DbResult result = m_Db.PopResultWait();
        const AuthProcessStatus status = std::visit(
            [&sender](const auto& value) -> AuthProcessStatus {
                using T = std::decay_t<decltype(value)>;
                if constexpr (std::is_same_v<T, AuthDb::AuthenticateResult>) {
                    return SendAuthResultMessage(sender, value);
                } else if constexpr (std::is_same_v<T, AuthDb::AuthenticateExtendedResult>) {
                    return SendAuthExtendedResultMessage(sender, value);
                } else {
                    return SendLockResultMessage(sender, value);
                }
            },
            result);
        if (status != AuthProcessStatus::Ok) {
            return status;
        }
        processed = std::bit_cast<std::int32_t>(
            std::bit_cast<std::uint32_t>(processed) + std::uint32_t{1});
        --remaining;
    }
    return AuthProcessStatus::Ok;
}

bool CGame::PushDBQuest(AuthDb::DbQuest quest)
{
    return m_Db.PushQuest(std::move(quest));
}

void CGame::PushServerInfo(AuthDb::ServerInfo info)
{
    m_Db.PushServerInfo(std::move(info));
}

AuthDbContext CGame::DatabaseContext() const
{
    return m_Db;
}

AuthMessageHandlers& CGame::MessageHandlers() noexcept
{
    return m_MessageHandlers;
}

const AuthMessageHandlers& CGame::MessageHandlers() const noexcept
{
    return m_MessageHandlers;
}

CMyNetServerAuth* CGame::NetServerAuth() noexcept
{
    return m_NetServerAuth.get();
}

const CMyNetServerAuth* CGame::NetServerAuth() const noexcept
{
    return m_NetServerAuth.get();
}

bool CGame::InitializeNetwork(const AuthNetworkConfig& config,
                              const std::filesystem::path& allowedClientsPath,
                              bool& allowedClientsLoaded,
                              std::error_code& error)
{
    ReleaseNetwork();
    m_NetworkTaskFailures = 0;

    m_NetServerAuth =
        std::make_unique<CMyNetServerAuth>(m_IoContext.get_executor(), AuthLegacyTickMs());
    allowedClientsLoaded = m_NetServerAuth->LoadAllowedClients(allowedClientsPath.string());

    if (!m_NetServerAuth->Host(config.hostPort,
                               std::nullopt,
                               kDefaultSocketType,
                               true,
                               error)) {
        return false;
    }

    if (const auto address = ResolveFirstLocalIPv4()) {
        const auto bytes = address->to_bytes();
        const IPv4Octets octets{bytes[0], bytes[1], bytes[2], bytes[3]};
        const std::string text = address->to_string();
        m_NetServerAuth->SetLocalIdentity(
            std::span<const std::uint8_t>(
                reinterpret_cast<const std::uint8_t*>(text.data()), text.size()),
            LegacyIPv4Word(octets));
    } else {
        m_NetServerAuth->SetLocalIdentity({}, 0U);
    }

    // Исходный CGame записывал эти поля после Host; backlog=10 уже не меняет
    // созданный listener, SendInterTime меняет timeout первого сообщения.
    m_NetServerAuth->ConfigureLimits(config.maxLoginServers,
                                     config.maxInFlightSends,
                                     config.permittedSendBytes,
                                     config.newAcceptTimeoutMs);
    m_NextAcceptAt = std::chrono::steady_clock::now();
    error.clear();
    return true;
}

void CGame::ReleaseNetwork()
{
    if (!m_NetServerAuth) {
        m_AcceptPending = false;
        m_AcceptCompletion.reset();
        return;
    }

    m_NetServerAuth->StopListening();
    static_cast<void>(m_NetServerAuth->QuitAllClients());

    while (m_AcceptPending || m_ActiveIoTasks != 0U || m_NetServerAuth->HasClients()) {
        PumpIo();
        m_AcceptCompletion.reset();

        if (m_NetServerAuth->HasClients()) {
            ServerSnapshot snapshot =
                m_NetServerAuth->ProcessCommandSnapshot(AuthLegacyTickMs());
            SpawnIoActions(std::move(snapshot.ioActions), m_NetServerAuth->CommandHandle());
        }

        if (m_AcceptPending || m_ActiveIoTasks != 0U || m_NetServerAuth->HasClients()) {
            std::this_thread::sleep_for(kMainLoopDelay);
        }
    }

    PumpIo();
    m_NetServerAuth.reset();
    m_AcceptPending = false;
    m_AcceptCompletion.reset();
}

void CGame::PumpIo()
{
    if (m_IoContext.stopped()) {
        m_IoContext.restart();
    }
    while (m_IoContext.poll_one() != 0U) {
    }
}

void CGame::HandleAcceptCompletion(std::uint32_t nowMs, AuthRuntimeStep& step)
{
    if (!m_AcceptCompletion || !m_NetServerAuth) {
        return;
    }

    AcceptedTransport completion = std::move(*m_AcceptCompletion);
    m_AcceptCompletion.reset();
    m_NextAcceptAt = std::chrono::steady_clock::now() + kAcceptThreadDelay;
    if (completion.error || !completion.socket) {
        return;
    }
    step.admissions.push_back(
        m_NetServerAuth->QueueAccepted(std::move(completion.socket), completion.peer, nowMs));
}

void CGame::StartAcceptIfDue()
{
    if (!m_NetServerAuth || m_AcceptPending ||
        std::chrono::steady_clock::now() < m_NextAcceptAt) {
        return;
    }

    switch (m_NetServerAuth->BeginAccept()) {
    case AcceptStart::NotListening:
        return;
    case AcceptStart::AtCapacity:
        m_NextAcceptAt = std::chrono::steady_clock::now() + kAcceptAtCapacityDelay;
        return;
    case AcceptStart::Pending:
        break;
    }

    m_AcceptPending = true;
    asio::co_spawn(
        m_IoContext,
        m_NetServerAuth->AcceptOne(),
        [this](std::exception_ptr exception, AcceptedTransport completion) {
            m_AcceptPending = false;
            if (exception) {
                ++m_NetworkTaskFailures;
                return;
            }
            m_AcceptCompletion = std::move(completion);
        });
}

void CGame::SpawnIoActions(std::vector<ServerIoAction> actions,
                           ServerCommandHandle commands)
{
    for (ServerIoAction& action : actions) {
        ++m_ActiveIoTasks;
        asio::co_spawn(
            m_IoContext,
            RunServerIoAction(std::move(action), commands),
            [this](std::exception_ptr exception) {
                if (exception) {
                    ++m_NetworkTaskFailures;
                }
                if (m_ActiveIoTasks != 0U) {
                    --m_ActiveIoTasks;
                }
            });
    }
}

CGame::ServerInfoUpdate CGame::UpdateServerInfo()
{
    ServerInfoUpdate result;
    if (!m_NetServerAuth) {
        result.status = AuthProcessStatus::NetworkNotInitialized;
        return result;
    }

    const ConfigReader config = m_Db.ConfigSnapshot();
    const std::uint32_t nowMs = AuthLegacyTickMs();
    std::lock_guard<std::mutex> lock(g_ServerInfoTimersMutex);
    if (!g_ServerInfoTimers.lastUpdateMs) {
        g_ServerInfoTimers.lastUpdateMs = nowMs;
    }
    if (!g_ServerInfoTimers.lastWriteMs) {
        g_ServerInfoTimers.lastWriteMs = nowMs;
    }

    result.requested =
        (*g_ServerInfoTimers.lastUpdateMs + config.UpdateServerInfoTimeMs()) < nowMs;
    if (result.requested) {
        const ServerCommandHandle sender = m_NetServerAuth->CommandHandle();
        CMessage request(0x000CF702);
        for (const std::int32_t socketId : m_MessageHandlers.LoginServerSocketIDs()) {
            if (std::holds_alternative<AuthSendMessageError>(
                    request.SendToLogin(sender, socketId))) {
                result.status = AuthProcessStatus::MessageBuildFailed;
                return result;
            }
        }
        g_ServerInfoTimers.lastUpdateMs = nowMs;
    }

    result.writeDue =
        (*g_ServerInfoTimers.lastWriteMs + config.WriteServerInfoTimeMs()) < nowMs;
    if (result.writeDue) {
        result.writeQueued = m_Db.PushQuest(AuthDb::WriteServerInfoQuest{});
        // Timer обновлялся даже при отказе переполненной DB queue.
        g_ServerInfoTimers.lastWriteMs = nowMs;
    }
    return result;
}

std::optional<asio::ip::address_v4> CGame::ResolveFirstLocalIPv4()
{
    asio::error_code error;
    const std::string host = asio::ip::host_name(error);
    if (error) {
        return std::nullopt;
    }

    asio::ip::tcp::resolver resolver(m_IoContext);
    const auto results = resolver.resolve(asio::ip::tcp::v4(), host, "0", error);
    if (error) {
        return std::nullopt;
    }
    for (const auto& entry : results) {
        if (entry.endpoint().address().is_v4()) {
            return entry.endpoint().address().to_v4();
        }
    }
    return std::nullopt;
}

std::uint32_t AuthLegacyTickMs() noexcept
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
    const auto fallback = std::chrono::steady_clock::now().time_since_epoch();
    return static_cast<std::uint32_t>(
        std::chrono::duration_cast<std::chrono::milliseconds>(fallback).count());
}
