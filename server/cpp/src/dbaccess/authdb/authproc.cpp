#include "authproc.h"

#include "../../auth/cgame.h"
#include "../../auth/configreader.h"

#include <algorithm>
#include <chrono>
#include <exception>
#include <ctime>
#include <mutex>
#include <system_error>
#include <thread>
#include <type_traits>
#include <utility>

namespace
{
constexpr auto kDbWorkerDelay = std::chrono::milliseconds(1);

void WriteSystemTime(std::array<std::uint8_t, 80>& output,
                     const AuthDb::LockUntil& time) noexcept
{
    const auto writeWord = [&output](std::size_t offset, std::uint16_t value) {
        output[offset] = static_cast<std::uint8_t>(value);
        output[offset + 1] = static_cast<std::uint8_t>(value >> 8U);
    };

    writeWord(0, time.year);
    writeWord(2, time.month);
    // offset +4 (day-of-week) исходный output оставлял нулевым.
    writeWord(6, time.day);
    writeWord(8, time.hour);
    writeWord(10, time.minute);
    writeWord(12, time.second);
    // offset +14 (milliseconds) остаётся нулевым.
}

AuthDb::AuthResultData BuildAuthResult(AuthDb::AuthQuestData request,
                                       std::int32_t databaseResult)
{
    return {
        .result = MapAuthResult(databaseResult),
        .account = std::move(request.account),
        .clientIp = request.clientIp,
        .clientSocketId = request.clientSocketId,
    };
}

AuthDb::AuthExResultData BuildAuthExtendedResult(
    AuthDb::AuthQuestData request,
    AuthExtendedDatabaseResult databaseResult)
{
    AuthDb::AuthExResultData result{
        .result = MapAuthExtendedResult(databaseResult.result),
        .account = std::move(request.account),
        .clientIp = request.clientIp,
        .clientSocketId = request.clientSocketId,
        .extra = {},
    };

    if (databaseResult.assure) {
        result.extra = *databaseResult.assure;
    } else if (databaseResult.suspended) {
        WriteSystemTime(result.extra, *databaseResult.suspended);
    }
    return result;
}

AuthLocalTime CurrentLocalTime() noexcept
{
    const auto now = std::chrono::system_clock::now();
    const auto milliseconds = std::chrono::duration_cast<std::chrono::milliseconds>(
                                  now.time_since_epoch()) %
                              std::chrono::seconds(1);
    const std::time_t timestamp = std::chrono::system_clock::to_time_t(now);
    std::tm local{};
#if defined(_WIN32)
    localtime_s(&local, &timestamp);
#else
    localtime_r(&timestamp, &local);
#endif
    return {
        .year = static_cast<std::uint16_t>(local.tm_year + 1900),
        .month = static_cast<std::uint16_t>(local.tm_mon + 1),
        .day = static_cast<std::uint16_t>(local.tm_mday),
        .hour = static_cast<std::uint16_t>(local.tm_hour),
        .minute = static_cast<std::uint16_t>(local.tm_min),
        .second = static_cast<std::uint16_t>(local.tm_sec),
        .milliseconds = static_cast<std::uint16_t>(milliseconds.count()),
    };
}

AuthDatabaseError MissingDatabaseError()
{
    return {
        std::make_error_code(std::errc::not_connected),
        "Auth database factory не создала MSSQL-compatible backend",
    };
}
}

DBCmdProc::DBCmdProc(std::unique_ptr<IAuthDatabase> database)
    : m_Database(std::move(database))
{
}

void DBCmdProc::ProcessQuest(AuthDbContext& context, AuthDb::DbQuest quest)
{
    std::visit(
        [this, &context](auto&& value) {
            using T = std::decay_t<decltype(value)>;
            if constexpr (std::is_same_v<T, AuthDb::AuthenticateQuest>) {
                ProcessAuthenticate(context,
                                    value.returnSocketId,
                                    std::move(value.request),
                                    false);
            } else if constexpr (std::is_same_v<T, AuthDb::AuthenticateExtendedQuest>) {
                ProcessAuthenticate(context,
                                    value.returnSocketId,
                                    std::move(value.request),
                                    true);
            } else if constexpr (std::is_same_v<T, AuthDb::LockQuest>) {
                ProcessLock(context,
                            value.returnSocketId,
                            std::move(value.request));
            } else if constexpr (std::is_same_v<T, AuthDb::WriteServerInfoQuest>) {
                ProcessWriteServerInfo(context);
            }
        },
        std::move(quest));
}

std::optional<AuthDatabaseNotice> DBCmdProc::PopNotice()
{
    if (m_Notices.empty()) {
        return std::nullopt;
    }
    AuthDatabaseNotice notice = std::move(m_Notices.front());
    m_Notices.pop_front();
    return notice;
}

void DBCmdProc::RefreshAuthConfig(const ConfigReader& config)
{
    if (m_Database) {
        m_Database->RefreshAuthConfig(config);
    }
}

void DBCmdProc::ProcessAuthenticate(AuthDbContext& context,
                                    std::int32_t returnSocketId,
                                    AuthDb::AuthQuestData request,
                                    bool extended)
{
    if (request.account.empty()) {
        return;
    }

    const ConfigReader config = context.ConfigSnapshot();
    const std::string_view procedure = config.GetDBSP(extended ? "sp_authex" : "sp_auth");
    if (procedure.empty()) {
        return;
    }

    if (!context.IsClientIpAllowed(request.clientIp)) {
        if (extended) {
            context.PushResult(AuthDb::AuthenticateExtendedResult{
                returnSocketId,
                BuildAuthExtendedResult(
                    std::move(request),
                    AuthExtendedDatabaseResult{-7, std::nullopt, std::nullopt})});
        } else {
            context.PushResult(AuthDb::AuthenticateResult{
                returnSocketId,
                BuildAuthResult(std::move(request), -7)});
        }
        return;
    }

    if (!m_Database) {
        PushNotice(extended ? AuthDatabaseOperation::AuthenticateExtended
                            : AuthDatabaseOperation::Authenticate,
                   MissingDatabaseError());
        if (extended) {
            context.PushResult(AuthDb::AuthenticateExtendedResult{
                returnSocketId,
                BuildAuthExtendedResult(
                    std::move(request),
                    AuthExtendedDatabaseResult{-2, std::nullopt, std::nullopt})});
        } else {
            context.PushResult(AuthDb::AuthenticateResult{
                returnSocketId,
                BuildAuthResult(std::move(request), -2)});
        }
        return;
    }

    if (extended) {
        auto databaseResult = m_Database->AuthenticateExtended(procedure, request);
        if (auto* error = std::get_if<AuthDatabaseError>(&databaseResult)) {
            PushNotice(AuthDatabaseOperation::AuthenticateExtended, std::move(*error));
            context.PushResult(AuthDb::AuthenticateExtendedResult{
                returnSocketId,
                BuildAuthExtendedResult(
                    std::move(request),
                    AuthExtendedDatabaseResult{-2, std::nullopt, std::nullopt})});
            return;
        }
        context.PushResult(AuthDb::AuthenticateExtendedResult{
            returnSocketId,
            BuildAuthExtendedResult(
                std::move(request),
                std::get<AuthExtendedDatabaseResult>(std::move(databaseResult)))});
        return;
    }

    auto databaseResult = m_Database->Authenticate(procedure, request);
    if (auto* error = std::get_if<AuthDatabaseError>(&databaseResult)) {
        PushNotice(AuthDatabaseOperation::Authenticate, std::move(*error));
        context.PushResult(AuthDb::AuthenticateResult{
            returnSocketId,
            BuildAuthResult(std::move(request), -2)});
        return;
    }
    context.PushResult(AuthDb::AuthenticateResult{
        returnSocketId,
        BuildAuthResult(std::move(request), std::get<std::int32_t>(databaseResult))});
}

void DBCmdProc::ProcessLock(AuthDbContext& context,
                            std::int32_t returnSocketId,
                            AuthDb::LockQuestData request)
{
    if (request.account.empty()) {
        return;
    }

    const ConfigReader config = context.ConfigSnapshot();
    const std::string_view procedure = config.GetDBSP("sp_lock");
    if (procedure.empty()) {
        return;
    }

    bool succeeded = false;
    if (!m_Database) {
        PushNotice(AuthDatabaseOperation::Lock, MissingDatabaseError());
    } else {
        auto databaseResult = m_Database->Lock(procedure, request);
        if (auto* error = std::get_if<AuthDatabaseError>(&databaseResult)) {
            PushNotice(AuthDatabaseOperation::Lock, std::move(*error));
        } else {
            succeeded = std::get<bool>(databaseResult);
        }
    }

    context.PushResult(AuthDb::LockResult{
        returnSocketId,
        AuthDb::LockResultData{std::move(request.account), succeeded}});
}

void DBCmdProc::ProcessWriteServerInfo(AuthDbContext& context)
{
    const ConfigReader config = context.ConfigSnapshot();
    const std::string_view procedure = config.GetDBSP("sp_writelog");
    if (procedure.empty()) {
        return;
    }

    if (m_Database) {
        m_Database->RefreshLogConfig(config);
    }
    // RVA 0x19270 фиксирует local time до атомарного pop_all.
    const AuthLocalTime loggedAt = CurrentLocalTime();
    std::deque<AuthDb::ServerInfo> entries = context.PopAllServerInfo();

    if (!m_Database) {
        PushNotice(AuthDatabaseOperation::WriteServerInfo, MissingDatabaseError());
        return;
    }
    if (auto error = m_Database->WriteServerInfo(procedure, loggedAt, std::move(entries))) {
        // Snapshot уже передан backend-у и не возвращается в очередь.
        PushNotice(AuthDatabaseOperation::WriteServerInfo, std::move(*error));
    }
}

void DBCmdProc::PushNotice(AuthDatabaseOperation operation, AuthDatabaseError error)
{
    m_Notices.push_back(AuthDatabaseNotice{operation, std::move(error)});
}

struct AuthDatabaseWorkers::Impl
{
    struct Worker
    {
        std::size_t index{};
        std::thread thread;
    };

    std::atomic_bool stopped{false};
    std::optional<AuthDbContext> context;
    std::vector<Worker> workers;
    std::mutex noticesMutex;
    std::deque<AuthDatabaseNotice> notices;
};

AuthDatabaseWorkers::AuthDatabaseWorkers()
    : m_Impl(std::make_unique<Impl>())
{
}

AuthDatabaseWorkers::~AuthDatabaseWorkers()
{
    static_cast<void>(StopAndJoin());
}

AuthDatabaseWorkerStartResult AuthDatabaseWorkers::Start(AuthDbContext context,
                                                          std::int32_t workerCount,
                                                          AuthDatabaseFactory factory)
{
    if (!m_Impl->workers.empty()) {
        return {AuthDatabaseWorkerStartStatus::AlreadyRunning, 0, {}};
    }

    const std::size_t count = workerCount > 0 ? static_cast<std::size_t>(workerCount) : 0U;
    if (count != 0U && !factory) {
        return {AuthDatabaseWorkerStartStatus::MissingBackend, 0, {}};
    }

    m_Impl->stopped.store(false, std::memory_order_release);
    m_Impl->context = context;

    for (std::size_t workerIndex = 0; workerIndex < count; ++workerIndex) {
        try {
            m_Impl->workers.push_back(Impl::Worker{
                workerIndex,
                std::thread([impl = m_Impl.get(), context, factory, workerIndex]() mutable {
                    std::unique_ptr<IAuthDatabase> database;
                    try {
                        if (factory) {
                            database = factory();
                        }
                    } catch (const std::exception& error) {
                        std::lock_guard<std::mutex> lock(impl->noticesMutex);
                        impl->notices.push_back(AuthDatabaseNotice{
                            AuthDatabaseOperation::Initialize,
                            {{}, error.what()}});
                        return;
                    } catch (...) {
                        std::lock_guard<std::mutex> lock(impl->noticesMutex);
                        impl->notices.push_back(AuthDatabaseNotice{
                            AuthDatabaseOperation::Initialize,
                            {{}, "неизвестная ошибка создания Auth DB backend"}});
                        return;
                    }

                    if (!database) {
                        std::lock_guard<std::mutex> lock(impl->noticesMutex);
                        impl->notices.push_back(AuthDatabaseNotice{
                            AuthDatabaseOperation::Initialize,
                            MissingDatabaseError()});
                        return;
                    }

                    try {
                        DBCmdProc processor(std::move(database));
                        while (!impl->stopped.load(std::memory_order_acquire)) {
                            processor.RefreshAuthConfig(context.ConfigSnapshot());
                            const std::uint32_t initialSize = context.QuestCount();
                            for (std::uint32_t index = 0; index < initialSize; ++index) {
                                auto quest = context.PopQuestUntilStopped(impl->stopped);
                                if (!quest) {
                                    break;
                                }
                                processor.ProcessQuest(context, std::move(*quest));
                                while (auto notice = processor.PopNotice()) {
                                    std::lock_guard<std::mutex> lock(impl->noticesMutex);
                                    impl->notices.push_back(std::move(*notice));
                                }
                            }
                            std::this_thread::sleep_for(kDbWorkerDelay);
                        }
                    } catch (const std::exception& error) {
                        std::lock_guard<std::mutex> lock(impl->noticesMutex);
                        impl->notices.push_back(AuthDatabaseNotice{
                            AuthDatabaseOperation::Initialize,
                            {{}, error.what()}});
                    } catch (...) {
                        std::lock_guard<std::mutex> lock(impl->noticesMutex);
                        impl->notices.push_back(AuthDatabaseNotice{
                            AuthDatabaseOperation::Initialize,
                            {{}, "неизвестная ошибка Auth DB worker"}});
                    }
                })});
        } catch (const std::system_error& error) {
            const std::error_code code = error.code();
            static_cast<void>(StopAndJoin());
            return {AuthDatabaseWorkerStartStatus::SpawnFailed, workerIndex, code};
        }
    }
    return {AuthDatabaseWorkerStartStatus::Ok, 0, {}};
}

std::vector<AuthDatabaseWorkerJoinError> AuthDatabaseWorkers::StopAndJoin()
{
    m_Impl->stopped.store(true, std::memory_order_release);
    if (m_Impl->context) {
        m_Impl->context->WakeQuestWaiters();
    }

    std::vector<AuthDatabaseWorkerJoinError> errors;
    for (Impl::Worker& worker : m_Impl->workers) {
        if (!worker.thread.joinable()) {
            errors.push_back({worker.index});
            continue;
        }
        try {
            worker.thread.join();
        } catch (...) {
            errors.push_back({worker.index});
        }
    }
    m_Impl->workers.clear();
    m_Impl->context.reset();
    return errors;
}

std::optional<AuthDatabaseNotice> AuthDatabaseWorkers::PopNotice()
{
    std::lock_guard<std::mutex> lock(m_Impl->noticesMutex);
    if (m_Impl->notices.empty()) {
        return std::nullopt;
    }
    AuthDatabaseNotice notice = std::move(m_Impl->notices.front());
    m_Impl->notices.pop_front();
    return notice;
}

bool AuthDatabaseWorkers::Running() const noexcept
{
    return !m_Impl->workers.empty();
}

std::int32_t MapAuthResult(std::int32_t databaseResult) noexcept
{
    switch (databaseResult) {
    case 0: return 0;
    case -10: return 12;
    case -9: return 11;
    case -8: return 10;
    case -7: return 5;
    case -6: return 8;
    case -5: return 9;
    case -4: return 6;
    case -3: return 3;
    case -1: return 2;
    default: return 1;
    }
}

std::int32_t MapAuthExtendedResult(std::int32_t databaseResult) noexcept
{
    switch (databaseResult) {
    case 0: return 0;
    case 1: return 7;
    case -7: return 5;
    case -3: return 3;
    case -1: return 2;
    default: return 1;
    }
}
