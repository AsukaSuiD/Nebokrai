#include "myadobase.h"

#include <iconv.h>
#include <sqlext.h>

#include <algorithm>
#include <cerrno>
#include <chrono>
#include <cstdio>
#include <ctime>
#include <string_view>
#include <utility>

std::string CMyAdoBase::m_strConnectionString;
std::string CMyAdoBase::m_strProvider;
std::string CMyAdoBase::m_strDataSource;
std::string CMyAdoBase::m_strInitialCatalog;
std::string CMyAdoBase::m_strUserID;
std::string CMyAdoBase::m_strPassword;
std::string CMyAdoBase::m_strConnectTimeout;
std::string CMyAdoBase::m_strIntegratedSecurity;

namespace
{
bool OdbcSucceeded(SQLRETURN result) noexcept
{
    return result == SQL_SUCCESS || result == SQL_SUCCESS_WITH_INFO;
}

std::string OdbcDiagnostic(SQLSMALLINT handleType,
                           SQLHANDLE handle,
                           std::string_view operation)
{
    SQLCHAR state[6]{};
    SQLINTEGER native = 0;
    SQLCHAR message[1024]{};
    SQLSMALLINT length = 0;
    const SQLRETURN diagnostic = SQLGetDiagRec(handleType,
                                                handle,
                                                1,
                                                state,
                                                &native,
                                                message,
                                                static_cast<SQLSMALLINT>(sizeof(message)),
                                                &length);
    std::string detail(operation);
    if (!OdbcSucceeded(diagnostic)) {
        detail += ": диагностические сведения ODBC недоступны";
        return detail;
    }

    detail += ": SQLSTATE=";
    detail += reinterpret_cast<const char*>(state);
    detail += ", системный код=" + std::to_string(native);
    if (length > 0) {
        const std::size_t copied = std::min<std::size_t>(
            static_cast<std::size_t>(length), sizeof(message) - 1U);
        detail += ", ";
        detail.append(reinterpret_cast<const char*>(message), copied);
    }
    return detail;
}

struct ConvertedText
{
    bool ok{};
    std::string value;
    std::string error;
};

ConvertedText Windows1251ToUtf8(std::string_view source)
{
    if (source.empty()) {
        return ConvertedText{.ok = true};
    }

    iconv_t converter = iconv_open("UTF-8", "WINDOWS-1251");
    if (converter == reinterpret_cast<iconv_t>(-1)) {
        return ConvertedText{.error = "iconv не поддерживает преобразование WINDOWS-1251 -> UTF-8"};
    }
    struct Closer
    {
        iconv_t converter;
        ~Closer() { iconv_close(converter); }
    } closer{converter};

    std::string output(source.size() * 4U + 4U, '\0');
    char* input = const_cast<char*>(source.data());
    std::size_t inputLeft = source.size();
    char* destination = output.data();
    std::size_t destinationLeft = output.size();
    errno = 0;
    if (iconv(converter,
              &input,
              &inputLeft,
              &destination,
              &destinationLeft) == static_cast<std::size_t>(-1)) {
        return ConvertedText{
            .error = "iconv не преобразовал WINDOWS-1251 в UTF-8, errno=" +
                     std::to_string(errno)};
    }
    output.resize(output.size() - destinationLeft);
    return ConvertedText{.ok = true, .value = std::move(output)};
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

bool BuildOdbcConnectionString(std::string_view server,
                               std::string_view database,
                               std::string_view user,
                               std::string_view password,
                               std::string& output,
                               std::string& error)
{
    const ConvertedText convertedServer = Windows1251ToUtf8(server);
    const ConvertedText convertedDatabase = Windows1251ToUtf8(database);
    const ConvertedText convertedUser = Windows1251ToUtf8(user);
    const ConvertedText convertedPassword = Windows1251ToUtf8(password);
    const ConvertedText* conversions[] = {
        &convertedServer, &convertedDatabase, &convertedUser, &convertedPassword};
    for (const ConvertedText* conversion : conversions) {
        if (!conversion->ok) {
            error = conversion->error;
            return false;
        }
    }

    output = "DRIVER={ODBC Driver 18 for SQL Server};SERVER=";
    output += EscapeOdbcValue(convertedServer.value);
    output += ";DATABASE=";
    output += EscapeOdbcValue(convertedDatabase.value);
    output += ";UID=";
    output += EscapeOdbcValue(convertedUser.value);
    output += ";PWD=";
    output += EscapeOdbcValue(convertedPassword.value);
    output += ";Encrypt=no;TrustServerCertificate=yes;";
    return true;
}
}

bool CMyAdoBase::Initialize(std::string provider,
                            std::string dataSource,
                            std::string initialCatalog,
                            std::string userId,
                            std::string password,
                            std::string connectTimeout,
                            std::string integratedSecurity)
{
    m_strProvider = std::move(provider);
    m_strDataSource = std::move(dataSource);
    m_strInitialCatalog = std::move(initialCatalog);
    m_strUserID = std::move(userId);
    m_strPassword = std::move(password);
    m_strConnectTimeout = std::move(connectTimeout);
    m_strIntegratedSecurity = std::move(integratedSecurity);

    // VERIFIED_DECOMPILE 0x00464CB0. ConnectTimeout и IntegratedSecurity
    // сохраняются, но в эту строку исходник их не добавляет.
    m_strConnectionString = "Provider=" + m_strProvider +
                            "; Server=" + m_strDataSource +
                            "; Database=" + m_strInitialCatalog +
                            "; UID=" + m_strUserID +
                            "; PWD=" + m_strPassword +
                            "; OLE DB Services=-1; Driver={SQL Server}";

    // CoInitialize(NULL) из старого владельца не имеет эквивалента в Linux и был
    // исключительно обвязкой COM.
    return true;
}

bool CMyAdoBase::Uninitalize() noexcept
{
    // Прямой владелец выполнял только CoUninitialize() + true.
    return true;
}

bool CMyAdoBase::CreateCn(Connection& connection)
{
    if (connection.created || connection.environment != SQL_NULL_HENV ||
        connection.handle != SQL_NULL_HDBC) {
        ReleaseCn(connection);
    }
    connection.lastError.clear();

    SQLRETURN result =
        SQLAllocHandle(SQL_HANDLE_ENV, SQL_NULL_HANDLE, &connection.environment);
    if (!OdbcSucceeded(result)) {
        connection.environment = SQL_NULL_HENV;
        connection.lastError = "SQLAllocHandle(SQL_HANDLE_ENV) завершился ошибкой";
        return false;
    }
    result = SQLSetEnvAttr(connection.environment,
                           SQL_ATTR_ODBC_VERSION,
                           reinterpret_cast<SQLPOINTER>(SQL_OV_ODBC3),
                           0);
    if (!OdbcSucceeded(result)) {
        connection.lastError = OdbcDiagnostic(
            SQL_HANDLE_ENV, connection.environment, "ошибка SQLSetEnvAttr(ODBC 3)");
        SQLFreeHandle(SQL_HANDLE_ENV, connection.environment);
        connection.environment = SQL_NULL_HENV;
        return false;
    }
    result = SQLAllocHandle(
        SQL_HANDLE_DBC, connection.environment, &connection.handle);
    if (!OdbcSucceeded(result)) {
        connection.lastError = OdbcDiagnostic(
            SQL_HANDLE_ENV, connection.environment, "ошибка SQLAllocHandle(SQL_HANDLE_DBC)");
        SQLFreeHandle(SQL_HANDLE_ENV, connection.environment);
        connection.environment = SQL_NULL_HENV;
        connection.handle = SQL_NULL_HDBC;
        return false;
    }
    connection.created = true;
    connection.open = false;
    return true;
}

bool CMyAdoBase::OpenCn(Connection& connection)
{
    connection.lastError.clear();
    if (!connection.created || connection.handle == SQL_NULL_HDBC) {
        connection.lastError = "OpenCn вызван без предварительного CreateCn";
        return false;
    }

    std::string odbcConnection;
    if (!BuildOdbcConnectionString(m_strDataSource,
                                   m_strInitialCatalog,
                                   m_strUserID,
                                   m_strPassword,
                                   odbcConnection,
                                   connection.lastError)) {
        return false;
    }

    const SQLRETURN result = SQLDriverConnect(
        connection.handle,
        nullptr,
        reinterpret_cast<SQLCHAR*>(odbcConnection.data()),
        SQL_NTS,
        nullptr,
        0,
        nullptr,
        SQL_DRIVER_NOPROMPT);
    if (!OdbcSucceeded(result)) {
        connection.lastError = OdbcDiagnostic(
            SQL_HANDLE_DBC, connection.handle, "ошибка SQLDriverConnect");
        return false;
    }
    connection.open = true;
    return true;
}

bool CMyAdoBase::ExecuteCn(const char* sql, Connection& connection)
{
    connection.lastError.clear();
    if (sql == nullptr || !connection.open || connection.handle == SQL_NULL_HDBC) {
        connection.lastError =
            sql == nullptr ? "ExecuteCn получил SQL=null"
                           : "ExecuteCn получил неоткрытое подключение";
        return false;
    }

    SQLHSTMT statement = SQL_NULL_HSTMT;
    SQLRETURN result =
        SQLAllocHandle(SQL_HANDLE_STMT, connection.handle, &statement);
    if (!OdbcSucceeded(result)) {
        connection.lastError = OdbcDiagnostic(
            SQL_HANDLE_DBC, connection.handle, "ошибка SQLAllocHandle(SQL_HANDLE_STMT)");
        return false;
    }

    result = SQLExecDirect(statement,
                           reinterpret_cast<SQLCHAR*>(const_cast<char*>(sql)),
                           SQL_NTS);
    if (!OdbcSucceeded(result) && result != SQL_NO_DATA) {
        connection.lastError =
            OdbcDiagnostic(SQL_HANDLE_STMT, statement, "ошибка SQLExecDirect");
        SQLFreeHandle(SQL_HANDLE_STMT, statement);
        return false;
    }

    SQLFreeHandle(SQL_HANDLE_STMT, statement);
    return true;
}

bool CMyAdoBase::CloseCn(Connection& connection)
{
    if (connection.handle == SQL_NULL_HDBC || !connection.open) {
        connection.open = false;
        return true;
    }

    const SQLRETURN result = SQLDisconnect(connection.handle);
    if (!OdbcSucceeded(result)) {
        connection.lastError =
            OdbcDiagnostic(SQL_HANDLE_DBC, connection.handle, "ошибка SQLDisconnect");
        return false;
    }
    connection.open = false;
    return true;
}

void CMyAdoBase::ReleaseCn(Connection& connection) noexcept
{
    // VERIFIED_ASSEMBLY 0x00464B40: ReleaseCn сначала вызывает CloseCn, затем
    // Release/null. Ошибка close не меняет дальнейший release.
    static_cast<void>(CloseCn(connection));
    if (connection.handle != SQL_NULL_HDBC) {
        SQLFreeHandle(SQL_HANDLE_DBC, connection.handle);
    }
    if (connection.environment != SQL_NULL_HENV) {
        SQLFreeHandle(SQL_HANDLE_ENV, connection.environment);
    }
    connection.handle = SQL_NULL_HDBC;
    connection.environment = SQL_NULL_HENV;
    connection.created = false;
    connection.open = false;
}

char* CMyAdoBase::GetTimeString(char* buffer, std::size_t capacity)
{
    if (buffer == nullptr || capacity == 0U) {
        return buffer;
    }

    const std::time_t now = std::chrono::system_clock::to_time_t(
        std::chrono::system_clock::now());
    std::tm local{};
#if defined(_WIN32)
    if (::localtime_s(&local, &now) != 0) {
        buffer[0] = '\0';
        return buffer;
    }
#else
    if (::localtime_r(&now, &local) == nullptr) {
        buffer[0] = '\0';
        return buffer;
    }
#endif

    // VERIFIED_ASSEMBLY 0x00464210: "%d-%d-%d %d:%d:%d", без zero-padding.
    static_cast<void>(std::snprintf(buffer,
                                    capacity,
                                    "%d-%d-%d %d:%d:%d",
                                    local.tm_year + 1900,
                                    local.tm_mon + 1,
                                    local.tm_mday,
                                    local.tm_hour,
                                    local.tm_min,
                                    local.tm_sec));
    return buffer;
}
