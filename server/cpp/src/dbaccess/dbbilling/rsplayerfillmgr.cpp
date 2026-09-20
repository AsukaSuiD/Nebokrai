#include "rsplayerfillmgr.h"

#include <spdlog/spdlog.h>

#include <algorithm>
#include <array>
#include <cctype>
#include <sstream>
#include <utility>

namespace Billing
{
namespace
{
using Nebokrai::Database::OdbcConnection;
using Nebokrai::Database::OdbcError;
using Nebokrai::Database::OdbcStatement;

std::optional<OdbcError> OpenDatabase(const BillingDatabaseConnection& settings,
                                      const Nebokrai::Database::MssqlConnectionOptions& options,
                                      OdbcConnection& connection)
{
    auto text = Nebokrai::Database::BuildMssqlConnectionString(
        settings.host, settings.database, settings.user, settings.password, options);
    if (auto* error = std::get_if<OdbcError>(&text)) {
        return *error;
    }
    if (auto error = connection.Create()) {
        return error;
    }
    return connection.Open(std::get<std::string>(text));
}

std::pair<SQLUSMALLINT, SQLUSMALLINT> FindColumns(SQLHSTMT statement)
{
    SQLSMALLINT count = 0;
    if (!Nebokrai::Database::OdbcSucceeded(SQLNumResultCols(statement, &count))) {
        return {};
    }
    SQLUSMALLINT accountColumn = 0;
    SQLUSMALLINT idColumn = 0;
    for (SQLUSMALLINT index = 1; index <= static_cast<SQLUSMALLINT>(count); ++index) {
        SQLCHAR name[128]{};
        SQLSMALLINT nameLength = 0;
        SQLSMALLINT type = 0;
        SQLULEN size = 0;
        SQLSMALLINT decimals = 0;
        SQLSMALLINT nullable = 0;
        if (!Nebokrai::Database::OdbcSucceeded(SQLDescribeCol(
                statement, index, name, sizeof(name), &nameLength, &type, &size,
                &decimals, &nullable))) {
            continue;
        }
        std::string column(reinterpret_cast<char*>(name),
                           static_cast<std::size_t>(nameLength));
        std::transform(column.begin(), column.end(), column.begin(), [](unsigned char ch) {
            return static_cast<char>(std::tolower(ch));
        });
        if (column == "account") accountColumn = index;
        if (column == "id") idColumn = index;
    }
    return {accountColumn, idColumn};
}
}

MssqlOdbcRsPlayerFill::MssqlOdbcRsPlayerFill(
    BillingDatabaseConnection settings,
    Nebokrai::Database::MssqlConnectionOptions transport)
    : m_Settings(std::move(settings)), m_Transport(std::move(transport))
{
}

std::vector<PlayerFillInfo> MssqlOdbcRsPlayerFill::GetPlayerFillLog()
{
    std::vector<PlayerFillInfo> entries;
    OdbcConnection connection;
    if (auto error = OpenDatabase(m_Settings, m_Transport, connection)) {
        spdlog::error("BillingServer: GetPlayerFillLog/connect: {}", error->detail);
        return entries;
    }
    OdbcStatement statement;
    if (auto error = statement.Create(connection)) {
        spdlog::error("BillingServer: GetPlayerFillLog/create: {}", error->detail);
        return entries;
    }
    if (auto error = statement.ExecuteDirect(
            "select top 50 * from TBL_NeedUpdate order by ID")) {
        spdlog::error("BillingServer: GetPlayerFillLog/execute: {}", error->detail);
        return entries;
    }
    const auto [accountColumn, idColumn] = FindColumns(statement.NativeHandle());
    if (accountColumn == 0 || idColumn == 0) {
        spdlog::error("BillingServer: TBL_NeedUpdate не содержит Account/ID");
        return entries;
    }

    while (true) {
        const SQLRETURN fetch = SQLFetch(statement.NativeHandle());
        if (fetch == SQL_NO_DATA) break;
        if (!Nebokrai::Database::OdbcSucceeded(fetch)) {
            spdlog::error("BillingServer: GetPlayerFillLog/fetch: {}",
                Nebokrai::Database::OdbcDiagnostic(
                    SQL_HANDLE_STMT, statement.NativeHandle(), "SQLFetch").detail);
            break;
        }
        std::array<char, 1024> account{};
        SQLLEN accountLength = 0;
        SQLINTEGER id = 0;
        SQLLEN idLength = 0;
        const SQLRETURN accountResult = SQLGetData(
            statement.NativeHandle(), accountColumn, SQL_C_CHAR, account.data(),
            account.size(), &accountLength);
        const SQLRETURN idResult = SQLGetData(
            statement.NativeHandle(), idColumn, SQL_C_SLONG, &id, sizeof(id), &idLength);
        if (!Nebokrai::Database::OdbcSucceeded(accountResult) ||
            !Nebokrai::Database::OdbcSucceeded(idResult) ||
            accountLength == SQL_NULL_DATA || idLength == SQL_NULL_DATA) {
            spdlog::error("BillingServer: строка TBL_NeedUpdate не содержит Account/ID");
            break;
        }
        const std::size_t visible = std::min<std::size_t>(
            accountLength < 0 ? 0U : static_cast<std::size_t>(accountLength),
            account.size() - 1U);
        entries.push_back(PlayerFillInfo{
            .playerAccount = std::vector<std::uint8_t>(account.begin(),
                                                       account.begin() + visible),
            .id = static_cast<std::int32_t>(id)});
    }
    return entries;
}

std::int32_t MssqlOdbcRsPlayerFill::DeletePlayerFillLog(
    const std::vector<PlayerFillInfo>& entries)
{
    if (entries.empty()) return 0;
    std::ostringstream ids;
    for (std::size_t index = 0; index < entries.size(); ++index) {
        if (index != 0) ids << ',';
        ids << entries[index].id;
    }
    const std::string sql =
        "delete from TBL_NeedUpdate where id in (" + ids.str() + ')';
    OdbcConnection connection;
    if (auto error = OpenDatabase(m_Settings, m_Transport, connection)) {
        spdlog::error("BillingServer: DeletePlayerFillLog/connect: {}", error->detail);
        return -2;
    }
    OdbcStatement statement;
    if (auto error = statement.Create(connection)) {
        spdlog::error("BillingServer: DeletePlayerFillLog/create: {}", error->detail);
        return -2;
    }
    if (auto error = statement.ExecuteDirect(sql)) {
        spdlog::error("BillingServer: DeletePlayerFillLog/execute: {}", error->detail);
        return -2;
    }
    return 0;
}

RsPlayerFillFactory MakeMssqlOdbcRsPlayerFillFactory(
    BillingDatabaseConnection settings,
    Nebokrai::Database::MssqlConnectionOptions transport)
{
    return [settings = std::move(settings), transport = std::move(transport)] {
        return std::make_unique<MssqlOdbcRsPlayerFill>(settings, transport);
    };
}
}
