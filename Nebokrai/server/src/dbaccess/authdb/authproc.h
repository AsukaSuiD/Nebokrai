#pragma once

#include "../../auth/dbqueue.h"

#include <array>
#include <atomic>
#include <cstddef>
#include <cstdint>
#include <deque>
#include <functional>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <system_error>
#include <variant>
#include <vector>

/*
 * Owner: dbaccess/authdb/authproc.cpp / authproc.h
 *
 * Точная пара: AuthServer/authserver.exe + AuthServer/authserver.pdb.
 * Исходный owner PDB: h:\fengyun\fy_russia\src\dbaccess\authdb\authproc.cpp.
 * RVA: ctor 0x00016E10, init 0x00016E20, do_auth 0x00017530,
 * do_auth_ex 0x00017CC0, do_lock 0x00018730, result builders
 * 0x00018DD0/0x00018F90/0x00019160, do_write_log 0x00019270,
 * process_quest 0x0001A360, worker entry 0x0001A910.
 *
 * MSSQL stored-procedure contract является частью baseline оригинала:
 * sp_auth/getAccInfo, sp_authex/GetAccount, sp_lock/suspendAccount и
 * sp_writelog/PutOnlineLog сохраняют параметры, output и result mapping.
 * Windows ADO/COM не является семантикой и заменяется IAuthDatabase. Конкретный
 * MSSQL driver/ODBC/TDS adapter реализует этот интерфейс отдельно; authproc и
 * CGame не зависят от транспортной библиотеки БД. Другие backend-ы здесь не
 * предполагаются и не влияют на восстановление baseline.
 *
 * Каждый worker сначала обновляет Auth connection settings из одного config
 * snapshot, затем отдельно читает 32-битный размер общей FIFO и выполняет ровно
 * столько ожидающих pop. После snapshot — 1 ms. WriteServerInfo отдельно
 * обновляет log settings, фиксирует одно local time до PopAll и при любой DB-
 * ошибке теряет уже вынутый coalesced snapshot, как оригинал.
 */

class AuthDbContext;
class ConfigReader;

struct AuthDatabaseError
{
    std::error_code code;
    std::string detail;
};

enum class AuthDatabaseOperation
{
    Initialize,
    Authenticate,
    AuthenticateExtended,
    Lock,
    WriteServerInfo,
};

struct AuthDatabaseNotice
{
    AuthDatabaseOperation operation{};
    AuthDatabaseError error;
};

struct AuthExtendedDatabaseResult
{
    std::int32_t result{};
    std::optional<std::array<std::uint8_t, 80>> assure;
    std::optional<AuthDb::LockUntil> suspended;
};

struct AuthLocalTime
{
    std::uint16_t year{};
    std::uint16_t month{};
    std::uint16_t day{};
    std::uint16_t hour{};
    std::uint16_t minute{};
    std::uint16_t second{};
    std::uint16_t milliseconds{};
};

class IAuthDatabase
{
public:
    virtual ~IAuthDatabase() = default;

    virtual void RefreshAuthConfig(const ConfigReader& config) = 0;
    virtual void RefreshLogConfig(const ConfigReader& config) = 0;

    [[nodiscard]] virtual std::variant<std::int32_t, AuthDatabaseError>
    Authenticate(std::string_view procedure, const AuthDb::AuthQuestData& request) = 0;

    [[nodiscard]] virtual std::variant<AuthExtendedDatabaseResult, AuthDatabaseError>
    AuthenticateExtended(std::string_view procedure,
                         const AuthDb::AuthQuestData& request) = 0;

    [[nodiscard]] virtual std::variant<bool, AuthDatabaseError>
    Lock(std::string_view procedure, const AuthDb::LockQuestData& request) = 0;

    [[nodiscard]] virtual std::optional<AuthDatabaseError>
    WriteServerInfo(std::string_view procedure,
                    const AuthLocalTime& loggedAt,
                    std::deque<AuthDb::ServerInfo> entries) = 0;
};

using AuthDatabaseFactory = std::function<std::unique_ptr<IAuthDatabase>()>;

class DBCmdProc
{
public:
    explicit DBCmdProc(std::unique_ptr<IAuthDatabase> database);

    void ProcessQuest(AuthDbContext& context, AuthDb::DbQuest quest);
    [[nodiscard]] std::optional<AuthDatabaseNotice> PopNotice();

    void RefreshAuthConfig(const ConfigReader& config);

private:
    void ProcessAuthenticate(AuthDbContext& context,
                             std::int32_t returnSocketId,
                             AuthDb::AuthQuestData request,
                             bool extended);
    void ProcessLock(AuthDbContext& context,
                     std::int32_t returnSocketId,
                     AuthDb::LockQuestData request);
    void ProcessWriteServerInfo(AuthDbContext& context);
    void PushNotice(AuthDatabaseOperation operation, AuthDatabaseError error);

    std::unique_ptr<IAuthDatabase> m_Database;
    std::deque<AuthDatabaseNotice> m_Notices;
};

enum class AuthDatabaseWorkerStartStatus
{
    Ok,
    AlreadyRunning,
    MissingBackend,
    SpawnFailed,
};

struct AuthDatabaseWorkerStartResult
{
    AuthDatabaseWorkerStartStatus status{AuthDatabaseWorkerStartStatus::Ok};
    std::size_t workerIndex{};
    std::error_code error;

    [[nodiscard]] constexpr explicit operator bool() const noexcept
    {
        return status == AuthDatabaseWorkerStartStatus::Ok;
    }
};

struct AuthDatabaseWorkerJoinError
{
    std::size_t workerIndex{};
};

class AuthDatabaseWorkers
{
public:
    AuthDatabaseWorkers();
    ~AuthDatabaseWorkers();

    AuthDatabaseWorkers(const AuthDatabaseWorkers&) = delete;
    AuthDatabaseWorkers& operator=(const AuthDatabaseWorkers&) = delete;

    [[nodiscard]] AuthDatabaseWorkerStartResult Start(AuthDbContext context,
                                                       std::int32_t workerCount,
                                                       AuthDatabaseFactory factory);
    [[nodiscard]] std::vector<AuthDatabaseWorkerJoinError> StopAndJoin();
    [[nodiscard]] std::optional<AuthDatabaseNotice> PopNotice();
    [[nodiscard]] bool Running() const noexcept;

private:
    struct Impl;
    std::unique_ptr<Impl> m_Impl;
};

[[nodiscard]] std::int32_t MapAuthResult(std::int32_t databaseResult) noexcept;
[[nodiscard]] std::int32_t MapAuthExtendedResult(std::int32_t databaseResult) noexcept;
