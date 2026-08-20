#pragma once

#include "authproc.h"

#include <memory>
#include <string>

/*
 * Технический владелец Linux: адаптер MSSQL/ODBC для восстановленного
 * IAuthDatabase. Это не исходный владелец Miracle и не новая семантика БД.
 * Контракт процедур, параметров, fallback и result mapping остаётся в
 * authproc.cpp; здесь заменяется только Windows ADO/COM transport.
 *
 * Реализация использует стандартный ODBC API (unixODBC на Linux). Каждый
 * Auth/AuthEx/Lock вызов открывает отдельное соединение, как оригинальный ADO-
 * путь. WriteServerInfo открывает одно log-соединение на уже снятый snapshot и
 * выполняет PutOnlineLog последовательно с одной временной меткой.
 *
 * Исходные ANSI bytes setup/account/password трактуются как Windows-1251 и
 * преобразуются в UTF-8 через системный iconv перед передачей Linux ODBC
 * driver. IPv4 остаётся dotted ASCII. Connection diagnostics не включают
 * connection string и credential values.
 *
 * Системная зависимость для будущего build target: unixODBC development
 * headers/libodbc и установленный MSSQL-compatible ODBC driver. На текущем NAS
 * эта dependency ещё не установлена; adapter не заменяется заглушкой ради
 * сборки.
 */
struct MssqlOdbcOptions
{
    std::string driver{"ODBC Driver 18 for SQL Server"};
    bool encrypt{false};
    bool trustServerCertificate{true};
};

class MssqlOdbcAuthDatabase final : public IAuthDatabase
{
public:
    explicit MssqlOdbcAuthDatabase(MssqlOdbcOptions options = {});
    ~MssqlOdbcAuthDatabase() override;

    MssqlOdbcAuthDatabase(const MssqlOdbcAuthDatabase&) = delete;
    MssqlOdbcAuthDatabase& operator=(const MssqlOdbcAuthDatabase&) = delete;

    void RefreshAuthConfig(const ConfigReader& config) override;
    void RefreshLogConfig(const ConfigReader& config) override;

    [[nodiscard]] std::variant<std::int32_t, AuthDatabaseError>
    Authenticate(std::string_view procedure, const AuthDb::AuthQuestData& request) override;

    [[nodiscard]] std::variant<AuthExtendedDatabaseResult, AuthDatabaseError>
    AuthenticateExtended(std::string_view procedure,
                         const AuthDb::AuthQuestData& request) override;

    [[nodiscard]] std::variant<bool, AuthDatabaseError>
    Lock(std::string_view procedure, const AuthDb::LockQuestData& request) override;

    [[nodiscard]] std::optional<AuthDatabaseError>
    WriteServerInfo(std::string_view procedure,
                    const AuthLocalTime& loggedAt,
                    std::deque<AuthDb::ServerInfo> entries) override;

private:
    struct Impl;
    std::unique_ptr<Impl> m_Impl;
};

[[nodiscard]] AuthDatabaseFactory MakeMssqlOdbcDatabaseFactory(MssqlOdbcOptions options = {});
