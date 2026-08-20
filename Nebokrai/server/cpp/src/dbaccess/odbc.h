#pragma once

#include <sql.h>
#include <sqlext.h>

#include <optional>
#include <string>
#include <string_view>
#include <system_error>
#include <variant>

/*
 * Технический слой `Nebokrai` без отдельного исходного владельца. Он заменяет
 * повторявшуюся в нескольких службах обвязку ADO/COM и unixODBC: RAII handles,
 * диагностику и безопасную строку подключения MSSQL. Имена процедур, порядок и
 * типы параметров, наборы результатов и их интерпретация остаются в исходных
 * владельцах БД.
 */
namespace Nebokrai::Database
{
struct OdbcError
{
    std::error_code code;
    std::string detail;
};

struct MssqlConnectionOptions
{
    std::string driver{"ODBC Driver 18 for SQL Server"};
    bool encrypt{false};
    bool trustServerCertificate{true};
};

[[nodiscard]] bool OdbcSucceeded(SQLRETURN result) noexcept;

[[nodiscard]] OdbcError OdbcDiagnostic(SQLSMALLINT handleType,
                                       SQLHANDLE handle,
                                       std::string_view operation);

[[nodiscard]] OdbcError OdbcLocalError(std::errc code, std::string detail);

[[nodiscard]] std::variant<std::string, OdbcError>
BuildMssqlConnectionString(std::string_view host,
                           std::string_view database,
                           std::string_view user,
                           std::string_view password,
                           const MssqlConnectionOptions& options = {});

class OdbcConnection
{
public:
    OdbcConnection() = default;
    ~OdbcConnection();

    OdbcConnection(const OdbcConnection&) = delete;
    OdbcConnection& operator=(const OdbcConnection&) = delete;

    [[nodiscard]] std::optional<OdbcError> Create();
    [[nodiscard]] std::optional<OdbcError> Open(std::string_view connectionString);
    [[nodiscard]] std::optional<OdbcError> Close();
    void Reset() noexcept;

    [[nodiscard]] bool IsCreated() const noexcept;
    [[nodiscard]] bool IsOpen() const noexcept;
    [[nodiscard]] SQLHDBC NativeHandle() const noexcept;

private:
    SQLHENV m_Environment{SQL_NULL_HENV};
    SQLHDBC m_Connection{SQL_NULL_HDBC};
    bool m_Open{};
};

class OdbcStatement
{
public:
    OdbcStatement() = default;
    ~OdbcStatement();

    OdbcStatement(const OdbcStatement&) = delete;
    OdbcStatement& operator=(const OdbcStatement&) = delete;

    [[nodiscard]] std::optional<OdbcError> Create(OdbcConnection& connection);
    [[nodiscard]] std::optional<OdbcError> Prepare(std::string_view sql);
    [[nodiscard]] std::optional<OdbcError> Execute();
    [[nodiscard]] std::optional<OdbcError> ExecuteDirect(std::string_view sql);
    [[nodiscard]] std::optional<OdbcError> FetchOne();
    [[nodiscard]] std::optional<OdbcError> CloseCursor();
    void Reset() noexcept;

    [[nodiscard]] SQLHSTMT NativeHandle() const noexcept;

private:
    SQLHDBC m_Connection{SQL_NULL_HDBC};
    SQLHSTMT m_Statement{SQL_NULL_HSTMT};
};
}
