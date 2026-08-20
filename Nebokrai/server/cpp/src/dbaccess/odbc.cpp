#include "odbc.h"

#include "../public/textcodec.h"

#include <algorithm>
#include <utility>

namespace Nebokrai::Database
{
namespace
{
std::error_code OdbcErrorCode(SQLINTEGER native) noexcept
{
    if (native == 0) {
        return std::make_error_code(std::errc::io_error);
    }
    return {static_cast<int>(native), std::generic_category()};
}

std::string EscapeOdbcValue(std::string_view value)
{
    std::string escaped;
    escaped.reserve(value.size() + 2U);
    escaped.push_back('{');
    for (const char ch : value) {
        escaped.push_back(ch);
        if (ch == '}') {
            escaped.push_back('}');
        }
    }
    escaped.push_back('}');
    return escaped;
}
}

bool OdbcSucceeded(SQLRETURN result) noexcept
{
    return result == SQL_SUCCESS || result == SQL_SUCCESS_WITH_INFO;
}

OdbcError OdbcDiagnostic(SQLSMALLINT handleType,
                         SQLHANDLE handle,
                         std::string_view operation)
{
    SQLCHAR state[6]{};
    SQLINTEGER native = 0;
    SQLCHAR message[1024]{};
    SQLSMALLINT messageLength = 0;
    const SQLRETURN diagnostic = SQLGetDiagRec(
        handleType,
        handle,
        1,
        state,
        &native,
        message,
        static_cast<SQLSMALLINT>(sizeof(message)),
        &messageLength);

    std::string detail(operation);
    if (!OdbcSucceeded(diagnostic)) {
        detail += ": диагностические сведения ODBC недоступны";
        return {OdbcErrorCode(native), std::move(detail)};
    }

    detail += ": SQLSTATE=";
    detail += reinterpret_cast<const char*>(state);
    detail += ", системный код=" + std::to_string(native);
    if (messageLength > 0) {
        const std::size_t copied = std::min<std::size_t>(
            static_cast<std::size_t>(messageLength), sizeof(message) - 1U);
        detail += ", ";
        detail.append(reinterpret_cast<const char*>(message), copied);
    }
    return {OdbcErrorCode(native), std::move(detail)};
}

OdbcError OdbcLocalError(std::errc code, std::string detail)
{
    return {std::make_error_code(code), std::move(detail)};
}

std::variant<std::string, OdbcError>
BuildMssqlConnectionString(std::string_view host,
                           std::string_view database,
                           std::string_view user,
                           std::string_view password,
                           const MssqlConnectionOptions& options)
{
    if (options.driver.empty()) {
        return OdbcLocalError(std::errc::invalid_argument,
                              "имя драйвера MSSQL ODBC не задано");
    }

    const auto convertedHost = Nebokrai::Windows1251ToUtf8(host);
    const auto convertedDatabase = Nebokrai::Windows1251ToUtf8(database);
    const auto convertedUser = Nebokrai::Windows1251ToUtf8(user);
    const auto convertedPassword = Nebokrai::Windows1251ToUtf8(password);
    const TextConversionResult* conversions[] = {
        &convertedHost, &convertedDatabase, &convertedUser, &convertedPassword};
    for (const TextConversionResult* conversion : conversions) {
        if (!*conversion) {
            return OdbcLocalError(std::errc::illegal_byte_sequence,
                                  conversion->error);
        }
    }

    std::string connection;
    connection.reserve(256U + host.size() + database.size() +
                       user.size() + password.size());
    connection += "DRIVER=" + EscapeOdbcValue(options.driver) + ';';
    connection += "SERVER=" + EscapeOdbcValue(*convertedHost.value) + ';';
    connection += "DATABASE=" + EscapeOdbcValue(*convertedDatabase.value) + ';';
    connection += "UID=" + EscapeOdbcValue(*convertedUser.value) + ';';
    connection += "PWD=" + EscapeOdbcValue(*convertedPassword.value) + ';';
    connection += options.encrypt ? "Encrypt=yes;" : "Encrypt=no;";
    connection += options.trustServerCertificate
                      ? "TrustServerCertificate=yes;"
                      : "TrustServerCertificate=no;";
    return connection;
}

OdbcConnection::~OdbcConnection()
{
    Reset();
}

std::optional<OdbcError> OdbcConnection::Create()
{
    Reset();
    SQLRETURN result = SQLAllocHandle(SQL_HANDLE_ENV, SQL_NULL_HANDLE, &m_Environment);
    if (!OdbcSucceeded(result)) {
        m_Environment = SQL_NULL_HENV;
        return OdbcLocalError(std::errc::io_error,
                              "SQLAllocHandle(SQL_HANDLE_ENV) завершился ошибкой");
    }

    result = SQLSetEnvAttr(m_Environment,
                           SQL_ATTR_ODBC_VERSION,
                           reinterpret_cast<SQLPOINTER>(SQL_OV_ODBC3),
                           0);
    if (!OdbcSucceeded(result)) {
        OdbcError error = OdbcDiagnostic(
            SQL_HANDLE_ENV, m_Environment, "SQLSetEnvAttr(ODBC 3)");
        Reset();
        return error;
    }

    result = SQLAllocHandle(SQL_HANDLE_DBC, m_Environment, &m_Connection);
    if (!OdbcSucceeded(result)) {
        OdbcError error = OdbcDiagnostic(
            SQL_HANDLE_ENV, m_Environment, "SQLAllocHandle(SQL_HANDLE_DBC)");
        Reset();
        return error;
    }
    return std::nullopt;
}

std::optional<OdbcError> OdbcConnection::Open(std::string_view connectionString)
{
    if (!IsCreated()) {
        if (auto error = Create()) {
            return error;
        }
    }
    if (m_Open) {
        return std::nullopt;
    }

    const SQLRETURN result = SQLDriverConnect(
        m_Connection,
        nullptr,
        reinterpret_cast<SQLCHAR*>(const_cast<char*>(connectionString.data())),
        static_cast<SQLSMALLINT>(connectionString.size()),
        nullptr,
        0,
        nullptr,
        SQL_DRIVER_NOPROMPT);
    if (!OdbcSucceeded(result)) {
        return OdbcDiagnostic(SQL_HANDLE_DBC, m_Connection, "SQLDriverConnect");
    }
    m_Open = true;
    return std::nullopt;
}

std::optional<OdbcError> OdbcConnection::Close()
{
    if (!m_Open || m_Connection == SQL_NULL_HDBC) {
        m_Open = false;
        return std::nullopt;
    }
    const SQLRETURN result = SQLDisconnect(m_Connection);
    if (!OdbcSucceeded(result)) {
        return OdbcDiagnostic(SQL_HANDLE_DBC, m_Connection, "SQLDisconnect");
    }
    m_Open = false;
    return std::nullopt;
}

void OdbcConnection::Reset() noexcept
{
    if (m_Connection != SQL_NULL_HDBC) {
        if (m_Open) {
            static_cast<void>(SQLDisconnect(m_Connection));
        }
        static_cast<void>(SQLFreeHandle(SQL_HANDLE_DBC, m_Connection));
    }
    if (m_Environment != SQL_NULL_HENV) {
        static_cast<void>(SQLFreeHandle(SQL_HANDLE_ENV, m_Environment));
    }
    m_Connection = SQL_NULL_HDBC;
    m_Environment = SQL_NULL_HENV;
    m_Open = false;
}

bool OdbcConnection::IsCreated() const noexcept
{
    return m_Environment != SQL_NULL_HENV && m_Connection != SQL_NULL_HDBC;
}

bool OdbcConnection::IsOpen() const noexcept
{
    return m_Open;
}

SQLHDBC OdbcConnection::NativeHandle() const noexcept
{
    return m_Connection;
}

OdbcStatement::~OdbcStatement()
{
    Reset();
}

std::optional<OdbcError> OdbcStatement::Create(OdbcConnection& connection)
{
    Reset();
    if (!connection.IsOpen()) {
        return OdbcLocalError(std::errc::not_connected,
                              "ODBC statement создан без открытого подключения");
    }
    m_Connection = connection.NativeHandle();
    const SQLRETURN result =
        SQLAllocHandle(SQL_HANDLE_STMT, m_Connection, &m_Statement);
    if (!OdbcSucceeded(result)) {
        m_Statement = SQL_NULL_HSTMT;
        return OdbcDiagnostic(
            SQL_HANDLE_DBC, m_Connection, "SQLAllocHandle(SQL_HANDLE_STMT)");
    }
    return std::nullopt;
}

std::optional<OdbcError> OdbcStatement::Prepare(std::string_view sql)
{
    if (m_Statement == SQL_NULL_HSTMT) {
        return OdbcLocalError(std::errc::not_connected,
                              "SQLPrepare вызван без ODBC statement");
    }
    const SQLRETURN result = SQLPrepare(
        m_Statement,
        reinterpret_cast<SQLCHAR*>(const_cast<char*>(sql.data())),
        static_cast<SQLINTEGER>(sql.size()));
    if (!OdbcSucceeded(result)) {
        return OdbcDiagnostic(SQL_HANDLE_STMT, m_Statement, "SQLPrepare");
    }
    return std::nullopt;
}

std::optional<OdbcError> OdbcStatement::Execute()
{
    const SQLRETURN result = SQLExecute(m_Statement);
    if (!OdbcSucceeded(result) && result != SQL_NO_DATA) {
        return OdbcDiagnostic(SQL_HANDLE_STMT, m_Statement, "SQLExecute");
    }
    return std::nullopt;
}

std::optional<OdbcError> OdbcStatement::ExecuteDirect(std::string_view sql)
{
    const SQLRETURN result = SQLExecDirect(
        m_Statement,
        reinterpret_cast<SQLCHAR*>(const_cast<char*>(sql.data())),
        static_cast<SQLINTEGER>(sql.size()));
    if (!OdbcSucceeded(result) && result != SQL_NO_DATA) {
        return OdbcDiagnostic(SQL_HANDLE_STMT, m_Statement, "SQLExecDirect");
    }
    return std::nullopt;
}

std::optional<OdbcError> OdbcStatement::FetchOne()
{
    const SQLRETURN result = SQLFetch(m_Statement);
    if (result == SQL_NO_DATA) {
        return OdbcLocalError(
            std::errc::no_message_available,
            "хранимая процедура MSSQL не вернула строку результата");
    }
    if (!OdbcSucceeded(result)) {
        return OdbcDiagnostic(SQL_HANDLE_STMT, m_Statement, "SQLFetch");
    }
    return std::nullopt;
}

std::optional<OdbcError> OdbcStatement::CloseCursor()
{
    const SQLRETURN result = SQLFreeStmt(m_Statement, SQL_CLOSE);
    if (!OdbcSucceeded(result)) {
        return OdbcDiagnostic(SQL_HANDLE_STMT, m_Statement, "SQLFreeStmt(SQL_CLOSE)");
    }
    return std::nullopt;
}

void OdbcStatement::Reset() noexcept
{
    if (m_Statement != SQL_NULL_HSTMT) {
        static_cast<void>(SQLFreeHandle(SQL_HANDLE_STMT, m_Statement));
    }
    m_Statement = SQL_NULL_HSTMT;
    m_Connection = SQL_NULL_HDBC;
}

SQLHSTMT OdbcStatement::NativeHandle() const noexcept
{
    return m_Statement;
}
}
