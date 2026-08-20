#pragma once

#include "appauth/message/message_func.h"
#include "configreader.h"
#include "dbqueue.h"
#include "kl_ipfilter.h"
#include "kl_multi_list.h"
#include "../dbaccess/authdb/authproc.h"
#include "../nets/netauth/mynetserver_auth.h"

#include <asio.hpp>

#include <atomic>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <deque>
#include <filesystem>
#include <memory>
#include <optional>
#include <shared_mutex>
#include <string>
#include <system_error>
#include <vector>

/*
 * Исходный владелец: authserver/src/cgame.cpp / cgame.h
 *
 * Точная пара: AuthServer/authserver.exe + AuthServer/authserver.pdb.
 * Исходные пути владельца в PDB:
 * Путь владельца в PDB: h:\fengyun\fy_russia\src\server\authserver\src\cgame.cpp/.h
 * Подтверждённые RVA: ProcessMessage 0x00002090, InitNetServer_Auth 0x00002AF0,
 * Release 0x000034C0, Send*Result 0x00004370/0x00004460/0x000045C0,
 * ProcessDBResult 0x00004E70, PushDBQuest 0x00005EC0,
 * UpdateServerInfo 0x00006310, MainLoop 0x00006890, GameThreadFunc 0x000068C0.
 *
 * CGame сохраняет исходные границы владельцев: config, DB quest/result FIFO,
 * coalesced ServerInfo, client deny-filter, Auth network и message handlers.
 * DB workers получают только AuthDbContext; network/handler state в их потоки
 * не передаётся. Конкретный DB driver находится за IAuthDatabase и обязан
 * реализовать baseline MSSQL stored-procedure contract из authproc.
 *
 * WinSock/IOCP и Windows threads заменены одним Asio io_context: accept и I/O
 * operations живут как co_spawn tasks, а game-loop каждые 1 ms выполняет
 * короткий poll + исходные snapshots. Это меняет технический scheduler, но не
 * порядок внутри command/message/result snapshots. timeGetTime заменён Linux
 * CLOCK_BOOTTIME с тем же 32-bit wrapping milliseconds.
 */

class AuthDbContext
{
public:
    explicit AuthDbContext(ConfigReader config = ConfigReader{});

    [[nodiscard]] ConfigReader ConfigSnapshot() const;
    [[nodiscard]] ConfigLoadResult LoadConfig(const std::filesystem::path& path);
    void ResetConfig();

    [[nodiscard]] bool PushQuest(AuthDb::DbQuest quest);
    [[nodiscard]] std::uint32_t QuestCount() const;
    [[nodiscard]] std::optional<AuthDb::DbQuest>
    PopQuestUntilStopped(const std::atomic_bool& stopped);
    void WakeQuestWaiters();

    void PushResult(AuthDb::DbResult result);
    [[nodiscard]] std::uint32_t ResultCount() const;
    [[nodiscard]] AuthDb::DbResult PopResultWait();

    void PushServerInfo(AuthDb::ServerInfo info);
    [[nodiscard]] std::deque<AuthDb::ServerInfo> PopAllServerInfo();

    void ReplaceClientForbidPatterns(std::vector<kl_net::IpPattern> patterns);
    [[nodiscard]] bool IsClientIpAllowed(std::uint32_t address) const;

private:
    struct State;
    std::shared_ptr<State> m_State;
};

enum class AuthInitializationNoticeType
{
    SetupDefaultsApplied,
    AllowedClientsUnavailable,
    ClientIpFilterUnavailable,
};

struct AuthInitializationNotice
{
    AuthInitializationNoticeType type{};
    ConfigLoadResult configResult{};
    kl_net::IpFilterLoadResult ipFilterResult{};
};

enum class AuthInitializationStatus
{
    Ok,
    NetworkFailed,
    DatabaseWorkersFailed,
};

struct AuthInitializationResult
{
    AuthInitializationStatus status{AuthInitializationStatus::Ok};
    std::error_code networkError;
    AuthDatabaseWorkerStartResult databaseWorkers;
    std::vector<AuthInitializationNotice> notices;

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return status == AuthInitializationStatus::Ok;
    }
};

struct AuthReleaseResult
{
    std::vector<AuthDatabaseWorkerJoinError> databaseWorkerErrors;
    std::size_t networkTaskFailures{};
};

enum class AuthProcessStatus
{
    Ok,
    NetworkNotInitialized,
    DispatchFailed,
    MessageBuildFailed,
};

struct AuthRuntimeStep
{
    AuthProcessStatus status{AuthProcessStatus::Ok};
    std::int32_t processedNetworkCommands{};
    std::int32_t processedMessages{};
    std::int32_t processedDatabaseResults{};
    bool serverInfoRequested{};
    bool serverInfoWriteDue{};
    bool serverInfoWriteQueued{};
    std::vector<AdmissionResult> admissions;
    std::vector<ServerSnapshotError> networkErrors;
    std::size_t networkTaskFailures{};
};

class CGame
{
public:
    explicit CGame(ConfigReader config = ConfigReader{},
                   AuthDatabaseFactory databaseFactory = {},
                   std::vector<kl_net::IpPattern> loginServerPatterns = {});
    ~CGame();

    CGame(const CGame&) = delete;
    CGame& operator=(const CGame&) = delete;

    [[nodiscard]] AuthInitializationResult
    InitializeRuntime(const std::filesystem::path& setupPath,
                      const std::filesystem::path& allowedClientsPath,
                      const std::filesystem::path& clientForbidPath);

    [[nodiscard]] AuthRuntimeStep RunNetworkTurn();
    [[nodiscard]] AuthRuntimeStep RunMainLoopTurn();
    [[nodiscard]] AuthReleaseResult ReleaseRuntime();

    [[nodiscard]] AuthProcessStatus ProcessMessages(std::int32_t& processed);
    [[nodiscard]] AuthProcessStatus ProcessDatabaseResults(std::int32_t& processed);

    [[nodiscard]] bool PushDBQuest(AuthDb::DbQuest quest);
    void PushServerInfo(AuthDb::ServerInfo info);

    [[nodiscard]] AuthDbContext DatabaseContext() const;
    [[nodiscard]] AuthMessageHandlers& MessageHandlers() noexcept;
    [[nodiscard]] const AuthMessageHandlers& MessageHandlers() const noexcept;
    [[nodiscard]] CMyNetServerAuth* NetServerAuth() noexcept;
    [[nodiscard]] const CMyNetServerAuth* NetServerAuth() const noexcept;

private:
    struct ServerInfoUpdate
    {
        AuthProcessStatus status{AuthProcessStatus::Ok};
        bool requested{};
        bool writeDue{};
        bool writeQueued{};
    };

    [[nodiscard]] bool InitializeNetwork(const AuthNetworkConfig& config,
                                         const std::filesystem::path& allowedClientsPath,
                                         bool& allowedClientsLoaded,
                                         std::error_code& error);
    void ReleaseNetwork();
    void PumpIo();
    void HandleAcceptCompletion(std::uint32_t nowMs, AuthRuntimeStep& step);
    void StartAcceptIfDue();
    void SpawnIoActions(std::vector<ServerIoAction> actions,
                        ServerCommandHandle commands);
    [[nodiscard]] ServerInfoUpdate UpdateServerInfo();
    [[nodiscard]] std::optional<asio::ip::address_v4> ResolveFirstLocalIPv4();

    asio::io_context m_IoContext;
    AuthDbContext m_Db;
    AuthDatabaseFactory m_DatabaseFactory;
    AuthDatabaseWorkers m_DbWorkers;
    std::unique_ptr<CMyNetServerAuth> m_NetServerAuth;
    AuthMessageHandlers m_MessageHandlers;

    bool m_AcceptPending{};
    std::optional<AcceptedTransport> m_AcceptCompletion;
    std::chrono::steady_clock::time_point m_NextAcceptAt{};
    std::size_t m_ActiveIoTasks{};
    std::size_t m_NetworkTaskFailures{};
};

[[nodiscard]] std::uint32_t AuthLegacyTickMs() noexcept;
