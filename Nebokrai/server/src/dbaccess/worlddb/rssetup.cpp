#include "rssetup.h"

#include "../odbc.h"

#include <sql.h>
#include <sqlext.h>

#include <algorithm>
#include <array>
#include <charconv>
#include <cctype>
#include <limits>
#include <mutex>
#include <optional>
#include <type_traits>
#include <utility>

using Nebokrai::Database::OdbcConnection;
using Nebokrai::Database::OdbcDiagnostic;
using Nebokrai::Database::OdbcStatement;
using Nebokrai::Database::OdbcSucceeded;

namespace
{
struct PreparedCommand {
    std::string sql;
    std::vector<const WorldDbValue*> parameters;
    std::string error;
};

PreparedCommand PrepareCommand(const WorldDbCommand& command)
{
    PreparedCommand prepared;
    prepared.sql.reserve(command.statement.size());
    for (std::size_t cursor = 0; cursor < command.statement.size();) {
        if (command.statement[cursor] != '@' || cursor + 2 >= command.statement.size() ||
            command.statement[cursor + 1] != 'P' ||
            !std::isdigit(static_cast<unsigned char>(command.statement[cursor + 2]))) {
            prepared.sql.push_back(command.statement[cursor++]);
            continue;
        }
        std::size_t end = cursor + 2;
        while (end < command.statement.size() &&
               std::isdigit(static_cast<unsigned char>(command.statement[end]))) ++end;
        std::size_t oneBased = 0;
        const auto parsed = std::from_chars(command.statement.data() + cursor + 2,
                                            command.statement.data() + end, oneBased);
        if (parsed.ec != std::errc{} || oneBased == 0 || oneBased > command.parameters.size()) {
            prepared.error = "SQL ссылается на отсутствующий параметр " +
                             command.statement.substr(cursor, end - cursor);
            return prepared;
        }
        prepared.sql.push_back('?');
        prepared.parameters.push_back(&command.parameters[oneBased - 1]);
        cursor = end;
    }
    return prepared;
}

bool BindParameters(SQLHSTMT statement,
                    const std::vector<const WorldDbValue*>& values,
                    std::vector<SQLLEN>& indicators,
                    std::string& error)
{
    indicators.assign(values.size(), 0);
    for (std::size_t index = 0; index < values.size(); ++index) {
        const WorldDbValue& value = *values[index];
        SQLSMALLINT cType = SQL_C_CHAR;
        SQLSMALLINT sqlType = SQL_VARCHAR;
        SQLULEN columnSize = 1;
        SQLPOINTER data = nullptr;
        SQLLEN bufferLength = 0;
        indicators[index] = 0;

        std::visit([&](const auto& typed) {
            using T = std::decay_t<decltype(typed)>;
            if constexpr (std::is_same_v<T, std::monostate>) {
                indicators[index] = SQL_NULL_DATA;
                sqlType = SQL_VARCHAR;
            } else if constexpr (std::is_same_v<T, std::int64_t>) {
                cType = SQL_C_SBIGINT; sqlType = SQL_BIGINT; columnSize = 19;
                data = const_cast<std::int64_t*>(&typed); bufferLength = sizeof(typed);
            } else if constexpr (std::is_same_v<T, std::uint64_t>) {
                cType = SQL_C_UBIGINT; sqlType = SQL_BIGINT; columnSize = 20;
                data = const_cast<std::uint64_t*>(&typed); bufferLength = sizeof(typed);
            } else if constexpr (std::is_same_v<T, double>) {
                cType = SQL_C_DOUBLE; sqlType = SQL_DOUBLE; columnSize = 15;
                data = const_cast<double*>(&typed); bufferLength = sizeof(typed);
            } else if constexpr (std::is_same_v<T, std::string>) {
                cType = SQL_C_CHAR; sqlType = SQL_VARCHAR;
                columnSize = std::max<std::size_t>(typed.size(), 1U);
                data = const_cast<char*>(typed.data()); bufferLength = static_cast<SQLLEN>(typed.size());
                indicators[index] = static_cast<SQLLEN>(typed.size());
            } else {
                cType = SQL_C_BINARY; sqlType = SQL_VARBINARY;
                columnSize = std::max<std::size_t>(typed.size(), 1U);
                data = const_cast<std::uint8_t*>(typed.data());
                bufferLength = static_cast<SQLLEN>(typed.size());
                indicators[index] = static_cast<SQLLEN>(typed.size());
            }
        }, value);

        const SQLRETURN bound = SQLBindParameter(
            statement, static_cast<SQLUSMALLINT>(index + 1), SQL_PARAM_INPUT,
            cType, sqlType, columnSize, 0, data, bufferLength, &indicators[index]);
        if (!OdbcSucceeded(bound)) {
            error = OdbcDiagnostic(SQL_HANDLE_STMT, statement, "SQLBindParameter").detail;
            return false;
        }
    }
    return true;
}

std::string ColumnName(SQLHSTMT statement, const SQLUSMALLINT column,
                       SQLSMALLINT& type)
{
    std::array<SQLCHAR, 256> name{};
    SQLSMALLINT length = 0, decimals = 0, nullable = 0;
    SQLULEN size = 0;
    if (!OdbcSucceeded(SQLDescribeCol(statement, column, name.data(), name.size(),
                                      &length, &type, &size, &decimals, &nullable))) return {};
    return {reinterpret_cast<const char*>(name.data()), static_cast<std::size_t>(length)};
}

WorldDbValue ReadColumn(SQLHSTMT statement, const SQLUSMALLINT column, const SQLSMALLINT type)
{
    SQLLEN length = 0;
    if (type == SQL_INTEGER || type == SQL_SMALLINT || type == SQL_TINYINT || type == SQL_BIGINT ||
        type == SQL_BIT) {
        std::int64_t value = 0;
        if (OdbcSucceeded(SQLGetData(statement, column, SQL_C_SBIGINT, &value, sizeof(value), &length)) &&
            length != SQL_NULL_DATA) return value;
        return std::monostate{};
    }
    if (type == SQL_REAL || type == SQL_FLOAT || type == SQL_DOUBLE || type == SQL_DECIMAL ||
        type == SQL_NUMERIC) {
        double value = 0;
        if (OdbcSucceeded(SQLGetData(statement, column, SQL_C_DOUBLE, &value, sizeof(value), &length)) &&
            length != SQL_NULL_DATA) return value;
        return std::monostate{};
    }
    const bool binary = type == SQL_BINARY || type == SQL_VARBINARY || type == SQL_LONGVARBINARY;
    std::vector<std::uint8_t> bytes;
    std::array<std::uint8_t, 4096> chunk{};
    while (true) {
        const SQLRETURN result = SQLGetData(statement, column, binary ? SQL_C_BINARY : SQL_C_CHAR,
                                            chunk.data(), chunk.size(), &length);
        if (length == SQL_NULL_DATA) return std::monostate{};
        if (!OdbcSucceeded(result) && result != SQL_SUCCESS_WITH_INFO) break;
        std::size_t copied = 0;
        if (length == SQL_NO_TOTAL || length >= static_cast<SQLLEN>(chunk.size())) {
            copied = chunk.size() - (binary ? 0U : 1U);
        } else {
            copied = static_cast<std::size_t>(std::max<SQLLEN>(length, 0));
        }
        bytes.insert(bytes.end(), chunk.begin(), chunk.begin() + copied);
        if (result == SQL_SUCCESS) break;
    }
    if (binary) return bytes;
    return std::string(bytes.begin(), bytes.end());
}
}

struct OdbcWorldDbExecutor::Impl {
    explicit Impl(std::string text) : connectionString(std::move(text)) {}
    std::string connectionString;
    OdbcConnection connection;
    std::string lastError;
    mutable std::recursive_mutex mutex;
    bool transactionOpen{};
};

OdbcWorldDbExecutor::OdbcWorldDbExecutor(std::string connectionString)
    : m_Impl(std::make_unique<Impl>(std::move(connectionString))) {}
OdbcWorldDbExecutor::~OdbcWorldDbExecutor() = default;

bool OdbcWorldDbExecutor::Open()
{
    std::scoped_lock lock(m_Impl->mutex);
    if (m_Impl->connection.IsOpen()) return true;
    if (const auto error = m_Impl->connection.Open(m_Impl->connectionString)) {
        m_Impl->lastError = error->detail;
        return false;
    }
    m_Impl->lastError.clear();
    return true;
}

bool OdbcWorldDbExecutor::IsOpen() const noexcept { return m_Impl->connection.IsOpen(); }
std::string_view OdbcWorldDbExecutor::LastError() const noexcept { return m_Impl->lastError; }

WorldDbResult OdbcWorldDbExecutor::Execute(const WorldDbCommand& command)
{
    std::scoped_lock lock(m_Impl->mutex);
    if (!m_Impl->connection.IsOpen()) return {false, {}, 0, "World DB не подключена"};
    PreparedCommand prepared = PrepareCommand(command);
    if (!prepared.error.empty()) return {false, {}, 0, std::move(prepared.error)};

    OdbcStatement statement;
    if (const auto error = statement.Create(m_Impl->connection)) return {false, {}, 0, error->detail};
    if (const auto error = statement.Prepare(prepared.sql)) return {false, {}, 0, error->detail};
    std::vector<SQLLEN> indicators;
    std::string bindError;
    if (!BindParameters(statement.NativeHandle(), prepared.parameters, indicators, bindError))
        return {false, {}, 0, std::move(bindError)};
    if (const auto error = statement.Execute()) return {false, {}, 0, error->detail};

    WorldDbResult result{true};
    while (true) {
        SQLLEN affected = 0;
        if (OdbcSucceeded(SQLRowCount(statement.NativeHandle(), &affected)) && affected > 0)
            result.affectedRows += static_cast<std::uint64_t>(affected);

        SQLSMALLINT columnCount = 0;
        if (!OdbcSucceeded(SQLNumResultCols(statement.NativeHandle(), &columnCount)))
            return {false, {}, 0, OdbcDiagnostic(SQL_HANDLE_STMT, statement.NativeHandle(), "SQLNumResultCols").detail};
        while (columnCount > 0) {
            const SQLRETURN fetched = SQLFetch(statement.NativeHandle());
            if (fetched == SQL_NO_DATA) break;
            if (!OdbcSucceeded(fetched))
                return {false, {}, 0, OdbcDiagnostic(SQL_HANDLE_STMT, statement.NativeHandle(), "SQLFetch").detail};
            WorldDbRow row;
            for (SQLUSMALLINT column = 1; column <= static_cast<SQLUSMALLINT>(columnCount); ++column) {
                SQLSMALLINT type = 0;
                std::string name = ColumnName(statement.NativeHandle(), column, type);
                if (!name.empty()) row.emplace(std::move(name), ReadColumn(statement.NativeHandle(), column, type));
            }
            result.rows.push_back(std::move(row));
        }
        const SQLRETURN more = SQLMoreResults(statement.NativeHandle());
        if (more == SQL_NO_DATA) break;
        if (!OdbcSucceeded(more))
            return {false, {}, 0, OdbcDiagnostic(SQL_HANDLE_STMT, statement.NativeHandle(), "SQLMoreResults").detail};
    }
    return result;
}

bool OdbcWorldDbExecutor::BeginTransaction()
{
    m_Impl->mutex.lock();
    if (m_Impl->transactionOpen || !m_Impl->connection.IsOpen() ||
        !OdbcSucceeded(SQLSetConnectAttr(m_Impl->connection.NativeHandle(), SQL_ATTR_AUTOCOMMIT,
                                         reinterpret_cast<SQLPOINTER>(SQL_AUTOCOMMIT_OFF), 0))) {
        m_Impl->mutex.unlock();
        return false;
    }
    m_Impl->transactionOpen = true;
    return true;
}

bool OdbcWorldDbExecutor::CommitTransaction()
{
    if (!m_Impl->transactionOpen) return false;
    const bool ended = m_Impl->connection.IsOpen() &&
        OdbcSucceeded(SQLEndTran(SQL_HANDLE_DBC, m_Impl->connection.NativeHandle(), SQL_COMMIT));
    if (!ended) {
        m_Impl->lastError = OdbcDiagnostic(SQL_HANDLE_DBC, m_Impl->connection.NativeHandle(),
                                           "SQLEndTran(SQL_COMMIT)").detail;
        return false;
    }
    const bool restored = OdbcSucceeded(SQLSetConnectAttr(
        m_Impl->connection.NativeHandle(), SQL_ATTR_AUTOCOMMIT,
        reinterpret_cast<SQLPOINTER>(SQL_AUTOCOMMIT_ON), 0));
    m_Impl->transactionOpen = false;
    m_Impl->mutex.unlock();
    if (!restored) {
        m_Impl->lastError = OdbcDiagnostic(SQL_HANDLE_DBC, m_Impl->connection.NativeHandle(),
                                           "SQLSetConnectAttr(SQL_AUTOCOMMIT_ON)").detail;
    }
    return restored;
}

bool OdbcWorldDbExecutor::RollbackTransaction()
{
    if (!m_Impl->transactionOpen) return false;
    const bool ended = m_Impl->connection.IsOpen() &&
        OdbcSucceeded(SQLEndTran(SQL_HANDLE_DBC, m_Impl->connection.NativeHandle(), SQL_ROLLBACK));
    const bool restored = OdbcSucceeded(SQLSetConnectAttr(
        m_Impl->connection.NativeHandle(), SQL_ATTR_AUTOCOMMIT,
        reinterpret_cast<SQLPOINTER>(SQL_AUTOCOMMIT_ON), 0));
    m_Impl->transactionOpen = false;
    m_Impl->mutex.unlock();
    return ended && restored;
}

WorldDbResult CRsSetup::Load(IWorldDbExecutor& database)
{
    return database.Execute({"SELECT TOP 1 playerID,LeaveWordID FROM csl_setup", {}});
}

WorldDbResult CRsSetup::SavePlayerId(IWorldDbExecutor& database, const std::int32_t id)
{
    return database.Execute({"UPDATE csl_setup SET playerID=@P1", {static_cast<std::int64_t>(id)}});
}

WorldDbResult CRsSetup::SaveLeaveWordId(IWorldDbExecutor& database, const std::int32_t id)
{
    return database.Execute({"UPDATE csl_setup SET LeaveWordID=@P1", {static_cast<std::int64_t>(id)}});
}
