#include "rscdkey_odbc.h"

#include "../../public/textcodec.h"

#include <spdlog/spdlog.h>

#include <sql.h>
#include <sqlext.h>

#include <algorithm>
#include <array>
#include <memory>
#include <stdexcept>
#include <string_view>
#include <utility>

namespace
{
using Nebokrai::Database::OdbcConnection;
using Nebokrai::Database::OdbcError;
using Nebokrai::Database::OdbcStatement;
using Nebokrai::Database::OdbcSucceeded;

[[noreturn]] void ThrowOdbc(const OdbcError& error)
{
    throw std::runtime_error(error.detail);
}

std::string SqlUtf8(std::string_view legacySql)
{
    const auto converted = Nebokrai::Windows1251ToUtf8(legacySql);
    if (!converted) {
        throw std::runtime_error(converted.error);
    }
    return *converted.value;
}

std::string LegacyText(std::string_view utf8)
{
    const auto converted = Nebokrai::ConvertTextEncoding(
        std::span<const char>(utf8.data(), utf8.size()),
        "UTF-8",
        "WINDOWS-1251");
    if (!converted) {
        throw std::runtime_error(converted.error);
    }
    return *converted.value;
}

bool IsBinaryType(SQLSMALLINT type) noexcept
{
    return type == SQL_BINARY || type == SQL_VARBINARY || type == SQL_LONGVARBINARY;
}

std::vector<std::uint8_t> ReadColumn(SQLHSTMT statement,
                                     SQLUSMALLINT column,
                                     SQLSMALLINT targetType)
{
    std::vector<std::uint8_t> value;
    std::array<std::uint8_t, 4096> chunk{};
    for (;;) {
        SQLLEN indicator = 0;
        const SQLRETURN result = SQLGetData(statement,
                                            column,
                                            targetType,
                                            chunk.data(),
                                            static_cast<SQLLEN>(chunk.size()),
                                            &indicator);
        if (indicator == SQL_NULL_DATA || result == SQL_NO_DATA) {
            break;
        }
        if (!OdbcSucceeded(result)) {
            ThrowOdbc(Nebokrai::Database::OdbcDiagnostic(
                SQL_HANDLE_STMT, statement, "SQLGetData"));
        }

        const std::size_t capacity = targetType == SQL_C_CHAR
            ? chunk.size() - 1U
            : chunk.size();
        std::size_t copied = capacity;
        if (result == SQL_SUCCESS) {
            if (targetType == SQL_C_CHAR) {
                copied = std::char_traits<char>::length(
                    reinterpret_cast<const char*>(chunk.data()));
            } else if (indicator != SQL_NO_TOTAL) {
                copied = std::min<std::size_t>(
                    capacity,
                    static_cast<std::size_t>(std::max<SQLLEN>(indicator, 0)));
            }
        }
        value.insert(value.end(), chunk.begin(), chunk.begin() + copied);
        if (result == SQL_SUCCESS) {
            break;
        }
    }
    return value;
}

struct OpenStatement
{
    std::unique_ptr<OdbcConnection> connection;
    std::unique_ptr<OdbcStatement> statement;
};

OpenStatement Open(const std::string& connectionString, const std::string& sql)
{
    OpenStatement opened{
        std::make_unique<OdbcConnection>(),
        std::make_unique<OdbcStatement>()};
    if (const auto error = opened.connection->Create()) {
        ThrowOdbc(*error);
    }
    if (const auto error = opened.connection->Open(connectionString)) {
        ThrowOdbc(*error);
    }
    if (const auto error = opened.statement->Create(*opened.connection)) {
        ThrowOdbc(*error);
    }
    if (const auto error = opened.statement->ExecuteDirect(SqlUtf8(sql))) {
        ThrowOdbc(*error);
    }
    return opened;
}
}

MssqlOdbcRsCdKeyDatabase::MssqlOdbcRsCdKeyDatabase(
    std::string host,
    std::string database,
    std::string user,
    std::string password,
    Nebokrai::Database::MssqlConnectionOptions options)
    : m_Host(std::move(host)),
      m_Database(std::move(database)),
      m_User(std::move(user)),
      m_Password(std::move(password)),
      m_Options(std::move(options))
{
}

std::string MssqlOdbcRsCdKeyDatabase::ConnectionString() const
{
    auto connection = Nebokrai::Database::BuildMssqlConnectionString(
        m_Host, m_Database, m_User, m_Password, m_Options);
    if (const auto* error = std::get_if<OdbcError>(&connection)) {
        ThrowOdbc(*error);
    }
    return std::get<std::string>(std::move(connection));
}

std::vector<CRsCDKey::Database::Row>
MssqlOdbcRsCdKeyDatabase::Query(const std::string& sql)
{
    OpenStatement opened = Open(ConnectionString(), sql);
    SQLSMALLINT columnCount = 0;
    if (!OdbcSucceeded(SQLNumResultCols(opened.statement->NativeHandle(), &columnCount))) {
        ThrowOdbc(Nebokrai::Database::OdbcDiagnostic(
            SQL_HANDLE_STMT, opened.statement->NativeHandle(), "SQLNumResultCols"));
    }

    struct Column
    {
        std::string name;
        SQLSMALLINT type{};
    };
    std::vector<Column> columns;
    columns.reserve(static_cast<std::size_t>(columnCount));
    for (SQLUSMALLINT index = 1; index <= static_cast<SQLUSMALLINT>(columnCount); ++index) {
        std::array<SQLCHAR, 256> name{};
        SQLSMALLINT nameLength = 0;
        SQLSMALLINT type = 0;
        SQLULEN size = 0;
        SQLSMALLINT decimalDigits = 0;
        SQLSMALLINT nullable = 0;
        const SQLRETURN described = SQLDescribeCol(opened.statement->NativeHandle(),
                                                   index,
                                                   name.data(),
                                                   static_cast<SQLSMALLINT>(name.size()),
                                                   &nameLength,
                                                   &type,
                                                   &size,
                                                   &decimalDigits,
                                                   &nullable);
        if (!OdbcSucceeded(described)) {
            ThrowOdbc(Nebokrai::Database::OdbcDiagnostic(
                SQL_HANDLE_STMT, opened.statement->NativeHandle(), "SQLDescribeCol"));
        }
        columns.push_back({
            LegacyText(std::string_view(reinterpret_cast<const char*>(name.data()),
                                        static_cast<std::size_t>(nameLength))),
            type});
    }

    std::vector<Row> rows;
    for (;;) {
        const SQLRETURN fetched = SQLFetch(opened.statement->NativeHandle());
        if (fetched == SQL_NO_DATA) {
            break;
        }
        if (!OdbcSucceeded(fetched)) {
            ThrowOdbc(Nebokrai::Database::OdbcDiagnostic(
                SQL_HANDLE_STMT, opened.statement->NativeHandle(), "SQLFetch"));
        }

        Row row;
        for (std::size_t i = 0; i < columns.size(); ++i) {
            const SQLUSMALLINT index = static_cast<SQLUSMALLINT>(i + 1U);
            if (IsBinaryType(columns[i].type)) {
                row.blobs.emplace(columns[i].name,
                                  ReadColumn(opened.statement->NativeHandle(), index, SQL_C_BINARY));
            } else {
                const auto bytes = ReadColumn(opened.statement->NativeHandle(), index, SQL_C_CHAR);
                row.columns.emplace(
                    columns[i].name,
                    LegacyText(std::string_view(reinterpret_cast<const char*>(bytes.data()),
                                                bytes.size())));
            }
        }
        rows.push_back(std::move(row));
    }
    return rows;
}

bool MssqlOdbcRsCdKeyDatabase::Execute(const std::string& sql)
{
    static_cast<void>(Open(ConnectionString(), sql));
    return true;
}

void MssqlOdbcRsCdKeyDatabase::PutStringToFile(const std::string& category,
                                                const std::string& message)
{
    spdlog::info("LoginServer [{}]: {}", category, message);
}

void MssqlOdbcRsCdKeyDatabase::PrintErr(const std::string& operation,
                                         const std::exception& error)
{
    spdlog::error("LoginServer: {}: {}", operation, error.what());
}
