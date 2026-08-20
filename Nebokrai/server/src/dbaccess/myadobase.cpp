#include "myadobase.h"

#include "odbc.h"

#include <chrono>
#include <cstdio>
#include <ctime>
#include <memory>
#include <utility>

std::string CMyAdoBase::m_strConnectionString;
std::string CMyAdoBase::m_strProvider;
std::string CMyAdoBase::m_strDataSource;
std::string CMyAdoBase::m_strInitialCatalog;
std::string CMyAdoBase::m_strUserID;
std::string CMyAdoBase::m_strPassword;
std::string CMyAdoBase::m_strConnectTimeout;
std::string CMyAdoBase::m_strIntegratedSecurity;

CMyAdoBase::Connection::Connection()
    : implementation(std::make_unique<Nebokrai::Database::OdbcConnection>())
{
}

CMyAdoBase::Connection::~Connection() = default;

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
    if (connection.created) {
        ReleaseCn(connection);
    }
    connection.lastError.clear();

    if (auto error = connection.implementation->Create()) {
        connection.lastError = std::move(error->detail);
        return false;
    }
    connection.created = true;
    connection.open = false;
    return true;
}

bool CMyAdoBase::OpenCn(Connection& connection)
{
    connection.lastError.clear();
    if (!connection.created) {
        connection.lastError = "OpenCn вызван без предварительного CreateCn";
        return false;
    }

    auto connectionText = Nebokrai::Database::BuildMssqlConnectionString(
        m_strDataSource, m_strInitialCatalog, m_strUserID, m_strPassword);
    if (auto* error = std::get_if<Nebokrai::Database::OdbcError>(&connectionText)) {
        connection.lastError = std::move(error->detail);
        return false;
    }

    if (auto error = connection.implementation->Open(
            std::get<std::string>(connectionText))) {
        connection.lastError = std::move(error->detail);
        return false;
    }
    connection.open = true;
    return true;
}

bool CMyAdoBase::ExecuteCn(const char* sql, Connection& connection)
{
    connection.lastError.clear();
    if (sql == nullptr || !connection.open) {
        connection.lastError =
            sql == nullptr ? "ExecuteCn получил SQL=null"
                           : "ExecuteCn получил неоткрытое подключение";
        return false;
    }

    Nebokrai::Database::OdbcStatement statement;
    if (auto error = statement.Create(*connection.implementation)) {
        connection.lastError = std::move(error->detail);
        return false;
    }
    if (auto error = statement.ExecuteDirect(sql)) {
        connection.lastError = std::move(error->detail);
        return false;
    }
    return true;
}

bool CMyAdoBase::ExecuteAccountEnterLog(std::string_view account,
                                        std::string_view enteredAt,
                                        std::string_view ip,
                                        Connection& connection)
{
    connection.lastError.clear();
    if (!connection.open) {
        connection.lastError =
            "ExecuteAccountEnterLog получил неоткрытое подключение";
        return false;
    }

    Nebokrai::Database::OdbcStatement statement;
    if (auto error = statement.Create(*connection.implementation)) {
        connection.lastError = std::move(error->detail);
        return false;
    }
    if (auto error = statement.Prepare(
            "INSERT INTO LogInfo(Account,AccountEnterTime,IP) VALUES(?,?,?)")) {
        connection.lastError = std::move(error->detail);
        return false;
    }

    std::string values[] = {
        std::string(account), std::string(enteredAt), std::string(ip)};
    SQLLEN lengths[] = {
        static_cast<SQLLEN>(values[0].size()),
        static_cast<SQLLEN>(values[1].size()),
        static_cast<SQLLEN>(values[2].size())};
    const SQLULEN legacySizes[] = {0x20U, 0x20U, 0x18U};
    for (SQLUSMALLINT index = 0; index < 3; ++index) {
        const SQLRETURN result = SQLBindParameter(
            statement.NativeHandle(),
            static_cast<SQLUSMALLINT>(index + 1U),
            SQL_PARAM_INPUT,
            SQL_C_CHAR,
            SQL_VARCHAR,
            legacySizes[index],
            0,
            values[index].data(),
            static_cast<SQLLEN>(values[index].size() + 1U),
            &lengths[index]);
        if (!Nebokrai::Database::OdbcSucceeded(result)) {
            connection.lastError = Nebokrai::Database::OdbcDiagnostic(
                SQL_HANDLE_STMT,
                statement.NativeHandle(),
                "SQLBindParameter(AccountEnterLog)").detail;
            return false;
        }
    }

    if (auto error = statement.Execute()) {
        connection.lastError = std::move(error->detail);
        return false;
    }
    return true;
}

bool CMyAdoBase::CloseCn(Connection& connection)
{
    if (!connection.open) {
        connection.open = false;
        return true;
    }

    if (auto error = connection.implementation->Close()) {
        connection.lastError = std::move(error->detail);
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
    connection.implementation->Reset();
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
