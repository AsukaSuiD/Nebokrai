#include "mssql_odbc.h"

#include "../odbc.h"
#include "../../auth/configreader.h"
#include "../../public/textcodec.h"

#include <algorithm>
#include <array>
#include <cctype>
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <system_error>
#include <utility>

namespace
{
bool OdbcSucceeded(SQLRETURN result) noexcept
{
    return Nebokrai::Database::OdbcSucceeded(result);
}

AuthDatabaseError OdbcDiagnostic(SQLSMALLINT handleType,
                                 SQLHANDLE handle,
                                 std::string_view operation)
{
    auto error = Nebokrai::Database::OdbcDiagnostic(handleType, handle, operation);
    return {std::move(error.code), std::move(error.detail)};
}

AuthDatabaseError LocalError(std::errc code, std::string detail)
{
    return {std::make_error_code(code), std::move(detail)};
}

std::variant<std::string, AuthDatabaseError>
Windows1251ToUtf8(std::string_view source)
{
    auto converted = Nebokrai::Windows1251ToUtf8(source);
    if (!converted) {
        return LocalError(std::errc::illegal_byte_sequence,
                          std::move(converted.error));
    }
    return std::move(*converted.value);
}

std::variant<std::string, AuthDatabaseError>
Windows1251ToUtf8(std::span<const std::uint8_t> source)
{
    return Windows1251ToUtf8(std::string_view(
        reinterpret_cast<const char*>(source.data()), source.size()));
}

std::variant<std::string, AuthDatabaseError>
BuildConnectionString(const AuthDatabaseSettings& settings,
                      const MssqlOdbcOptions& options)
{
    auto result = Nebokrai::Database::BuildMssqlConnectionString(
        settings.host,
        settings.database,
        settings.user,
        settings.password,
        Nebokrai::Database::MssqlConnectionOptions{
            .driver = options.driver,
            .encrypt = options.encrypt,
            .trustServerCertificate = options.trustServerCertificate});
    if (auto* error = std::get_if<Nebokrai::Database::OdbcError>(&result)) {
        return AuthDatabaseError{error->code, std::move(error->detail)};
    }
    return std::get<std::string>(std::move(result));
}

std::variant<std::string, AuthDatabaseError> QuoteProcedure(std::string_view procedure)
{
    if (procedure.empty()) {
        return LocalError(std::errc::invalid_argument,
                          "пустое имя хранимой процедуры MSSQL");
    }

    std::string quoted;
    std::size_t offset = 0;
    while (offset < procedure.size()) {
        const std::size_t separator = procedure.find('.', offset);
        const std::size_t end =
            separator == std::string_view::npos ? procedure.size() : separator;
        const std::string_view part = procedure.substr(offset, end - offset);
        if (part.empty() ||
            !std::all_of(part.begin(), part.end(), [](unsigned char ch) {
                return std::isalnum(ch) != 0 || ch == '_';
            })) {
            return LocalError(std::errc::invalid_argument,
                              "имя хранимой процедуры MSSQL содержит недопустимый идентификатор");
        }
        if (!quoted.empty()) {
            quoted.push_back('.');
        }
        quoted.push_back('[');
        quoted.append(part);
        quoted.push_back(']');
        if (separator == std::string_view::npos) {
            break;
        }
        offset = separator + 1U;
    }
    return quoted;
}

std::string LegacyIpv4Text(std::uint32_t address)
{
    return std::to_string(address & 0xFFU) + '.' +
           std::to_string((address >> 8U) & 0xFFU) + '.' +
           std::to_string((address >> 16U) & 0xFFU) + '.' +
           std::to_string((address >> 24U) & 0xFFU);
}

SQL_TIMESTAMP_STRUCT ToTimestamp(const AuthDb::LockUntil& value) noexcept
{
    SQL_TIMESTAMP_STRUCT timestamp{};
    timestamp.year = static_cast<SQLSMALLINT>(value.year);
    timestamp.month = static_cast<SQLUSMALLINT>(value.month);
    timestamp.day = static_cast<SQLUSMALLINT>(value.day);
    timestamp.hour = static_cast<SQLUSMALLINT>(value.hour);
    timestamp.minute = static_cast<SQLUSMALLINT>(value.minute);
    timestamp.second = static_cast<SQLUSMALLINT>(value.second);
    timestamp.fraction = 0;
    return timestamp;
}

SQL_TIMESTAMP_STRUCT ToTimestamp(const AuthLocalTime& value) noexcept
{
    SQL_TIMESTAMP_STRUCT timestamp{};
    timestamp.year = static_cast<SQLSMALLINT>(value.year);
    timestamp.month = static_cast<SQLUSMALLINT>(value.month);
    timestamp.day = static_cast<SQLUSMALLINT>(value.day);
    timestamp.hour = static_cast<SQLUSMALLINT>(value.hour);
    timestamp.minute = static_cast<SQLUSMALLINT>(value.minute);
    timestamp.second = static_cast<SQLUSMALLINT>(value.second);
    timestamp.fraction = static_cast<SQLUINTEGER>(value.milliseconds) * 1'000'000U;
    return timestamp;
}

struct OdbcConnection
{
    Nebokrai::Database::OdbcConnection implementation;

    std::optional<AuthDatabaseError> Open(std::string_view connectionString)
    {
        if (auto error = implementation.Open(connectionString)) {
            return AuthDatabaseError{std::move(error->code), std::move(error->detail)};
        }
        return std::nullopt;
    }

    [[nodiscard]] SQLHDBC Handle() const noexcept
    {
        return implementation.NativeHandle();
    }
};

struct OdbcStatement
{
    Nebokrai::Database::OdbcStatement implementation;

    std::optional<AuthDatabaseError> Prepare(OdbcConnection& connection,
                                             std::string_view sql)
    {
        if (auto error = implementation.Create(connection.implementation)) {
            return AuthDatabaseError{std::move(error->code), std::move(error->detail)};
        }
        if (auto error = implementation.Prepare(sql)) {
            return AuthDatabaseError{std::move(error->code), std::move(error->detail)};
        }
        return std::nullopt;
    }

    std::optional<AuthDatabaseError> Execute()
    {
        if (auto error = implementation.Execute()) {
            return AuthDatabaseError{std::move(error->code), std::move(error->detail)};
        }
        return std::nullopt;
    }

    std::optional<AuthDatabaseError> FetchOne()
    {
        if (auto error = implementation.FetchOne()) {
            return AuthDatabaseError{std::move(error->code), std::move(error->detail)};
        }
        return std::nullopt;
    }

    std::optional<AuthDatabaseError> CloseCursor()
    {
        if (auto error = implementation.CloseCursor()) {
            return AuthDatabaseError{std::move(error->code), std::move(error->detail)};
        }
        return std::nullopt;
    }

    [[nodiscard]] SQLHSTMT Handle() const noexcept
    {
        return implementation.NativeHandle();
    }
};

struct TextBinding
{
    std::string value;
    SQLLEN indicator{SQL_NTS};
};

std::optional<AuthDatabaseError>
BindText(SQLHSTMT statement, SQLUSMALLINT index, TextBinding& binding)
{
    const SQLULEN columnSize = static_cast<SQLULEN>(
        std::max<std::size_t>(200U, binding.value.size()));
    const SQLRETURN result = SQLBindParameter(statement,
                                              index,
                                              SQL_PARAM_INPUT,
                                              SQL_C_CHAR,
                                              SQL_VARCHAR,
                                              columnSize,
                                              0,
                                              binding.value.data(),
                                              static_cast<SQLLEN>(binding.value.size() + 1U),
                                              &binding.indicator);
    if (!OdbcSucceeded(result)) {
        return OdbcDiagnostic(SQL_HANDLE_STMT, statement, "SQLBindParameter(text)");
    }
    return std::nullopt;
}

struct IntBinding
{
    SQLINTEGER value{};
    SQLLEN indicator{};
};

std::optional<AuthDatabaseError>
BindInt(SQLHSTMT statement, SQLUSMALLINT index, IntBinding& binding)
{
    const SQLRETURN result = SQLBindParameter(statement,
                                              index,
                                              SQL_PARAM_INPUT,
                                              SQL_C_SLONG,
                                              SQL_INTEGER,
                                              0,
                                              0,
                                              &binding.value,
                                              0,
                                              &binding.indicator);
    if (!OdbcSucceeded(result)) {
        return OdbcDiagnostic(SQL_HANDLE_STMT, statement, "SQLBindParameter(int)");
    }
    return std::nullopt;
}

struct TimestampBinding
{
    SQL_TIMESTAMP_STRUCT value{};
    SQLLEN indicator{static_cast<SQLLEN>(sizeof(SQL_TIMESTAMP_STRUCT))};
};

std::optional<AuthDatabaseError>
BindTimestamp(SQLHSTMT statement, SQLUSMALLINT index, TimestampBinding& binding)
{
    const SQLRETURN result = SQLBindParameter(statement,
                                              index,
                                              SQL_PARAM_INPUT,
                                              SQL_C_TYPE_TIMESTAMP,
                                              SQL_TYPE_TIMESTAMP,
                                              23,
                                              3,
                                              &binding.value,
                                              0,
                                              &binding.indicator);
    if (!OdbcSucceeded(result)) {
        return OdbcDiagnostic(SQL_HANDLE_STMT, statement, "SQLBindParameter(timestamp)");
    }
    return std::nullopt;
}

std::variant<std::int32_t, AuthDatabaseError>
ReadInt(SQLHSTMT statement, SQLUSMALLINT column)
{
    SQLINTEGER value = 0;
    SQLLEN indicator = 0;
    const SQLRETURN result = SQLGetData(statement,
                                        column,
                                        SQL_C_SLONG,
                                        &value,
                                        sizeof(value),
                                        &indicator);
    if (!OdbcSucceeded(result)) {
        return OdbcDiagnostic(SQL_HANDLE_STMT, statement, "SQLGetData(int)");
    }
    if (indicator == SQL_NULL_DATA) {
        return LocalError(std::errc::no_message_available,
                          "хранимая процедура MSSQL вернула NULL вместо целого результата");
    }
    return static_cast<std::int32_t>(value);
}

std::variant<std::optional<std::array<std::uint8_t, 80>>, AuthDatabaseError>
ReadOptionalBinary80(SQLHSTMT statement, SQLUSMALLINT column)
{
    std::array<std::uint8_t, 80> value{};
    SQLLEN indicator = 0;
    const SQLRETURN result = SQLGetData(statement,
                                        column,
                                        SQL_C_BINARY,
                                        value.data(),
                                        static_cast<SQLLEN>(value.size()),
                                        &indicator);
    if (!OdbcSucceeded(result)) {
        return OdbcDiagnostic(SQL_HANDLE_STMT, statement, "SQLGetData(binary80)");
    }
    if (indicator == SQL_NULL_DATA) {
        return std::optional<std::array<std::uint8_t, 80>>{};
    }
    if (indicator != static_cast<SQLLEN>(value.size())) {
        return LocalError(std::errc::message_size,
                          "@Assure имеет длину, отличную от 80 байт");
    }
    return std::optional<std::array<std::uint8_t, 80>>(value);
}

std::variant<std::optional<AuthDb::LockUntil>, AuthDatabaseError>
ReadOptionalSuspended(SQLHSTMT statement)
{
    std::array<std::optional<std::int32_t>, 6> values;
    for (SQLUSMALLINT index = 0; index < values.size(); ++index) {
        SQLINTEGER value = 0;
        SQLLEN indicator = 0;
        const SQLRETURN result = SQLGetData(statement,
                                            static_cast<SQLUSMALLINT>(3U + index),
                                            SQL_C_SLONG,
                                            &value,
                                            sizeof(value),
                                            &indicator);
        if (!OdbcSucceeded(result)) {
            return OdbcDiagnostic(SQL_HANDLE_STMT,
                                  statement,
                                  "SQLGetData(suspended datepart)");
        }
        if (indicator != SQL_NULL_DATA) {
            values[index] = static_cast<std::int32_t>(value);
        }
    }

    if (std::all_of(values.begin(), values.end(), [](const auto& value) {
            return !value.has_value();
        })) {
        return std::optional<AuthDb::LockUntil>{};
    }
    if (std::any_of(values.begin(), values.end(), [](const auto& value) {
            return !value.has_value();
        })) {
        return LocalError(std::errc::invalid_argument,
                          "@suspended вернул неполный набор частей даты");
    }

    return std::optional<AuthDb::LockUntil>(AuthDb::LockUntil{
        static_cast<std::uint16_t>(*values[0]),
        static_cast<std::uint16_t>(*values[1]),
        static_cast<std::uint16_t>(*values[2]),
        static_cast<std::uint16_t>(*values[3]),
        static_cast<std::uint16_t>(*values[4]),
        static_cast<std::uint16_t>(*values[5]),
    });
}

std::optional<AuthDatabaseError>
DrainAllResults(SQLHSTMT statement)
{
    for (;;) {
        const SQLRETURN more = SQLMoreResults(statement);
        if (more == SQL_NO_DATA) {
            return std::nullopt;
        }
        if (!OdbcSucceeded(more)) {
            return OdbcDiagnostic(SQL_HANDLE_STMT, statement, "SQLMoreResults");
        }
    }
}

std::optional<AuthDatabaseError>
AdvanceToFirstRowset(SQLHSTMT statement)
{
    SQLSMALLINT columns = 0;
    for (;;) {
        const SQLRETURN countResult = SQLNumResultCols(statement, &columns);
        if (!OdbcSucceeded(countResult)) {
            return OdbcDiagnostic(SQL_HANDLE_STMT, statement, "SQLNumResultCols");
        }
        if (columns > 0) {
            return std::nullopt;
        }

        const SQLRETURN more = SQLMoreResults(statement);
        if (more == SQL_NO_DATA) {
            return LocalError(std::errc::no_message_available,
                              "хранимая процедура MSSQL не вернула результат SELECT");
        }
        if (!OdbcSucceeded(more)) {
            return OdbcDiagnostic(SQL_HANDLE_STMT, statement, "SQLMoreResults");
        }
    }
}
}

struct MssqlOdbcAuthDatabase::Impl
{
    explicit Impl(MssqlOdbcOptions value)
        : options(std::move(value))
    {
    }

    MssqlOdbcOptions options;
    AuthDatabaseSettings authSettings;
    AuthDatabaseSettings logSettings;
};

MssqlOdbcAuthDatabase::MssqlOdbcAuthDatabase(MssqlOdbcOptions options)
    : m_Impl(std::make_unique<Impl>(std::move(options)))
{
}

MssqlOdbcAuthDatabase::~MssqlOdbcAuthDatabase() = default;

void MssqlOdbcAuthDatabase::RefreshAuthConfig(const ConfigReader& config)
{
    m_Impl->authSettings = config.AuthDatabase();
}

void MssqlOdbcAuthDatabase::RefreshLogConfig(const ConfigReader& config)
{
    m_Impl->logSettings = config.LogDatabase();
}

std::variant<std::int32_t, AuthDatabaseError>
MssqlOdbcAuthDatabase::Authenticate(std::string_view procedure,
                                    const AuthDb::AuthQuestData& request)
{
    auto connectionText = BuildConnectionString(m_Impl->authSettings, m_Impl->options);
    if (auto* error = std::get_if<AuthDatabaseError>(&connectionText)) {
        return *error;
    }
    auto quotedProcedure = QuoteProcedure(procedure);
    if (auto* error = std::get_if<AuthDatabaseError>(&quotedProcedure)) {
        return *error;
    }
    auto account = Windows1251ToUtf8(std::span<const std::uint8_t>(request.account.data(), request.account.size()));
    if (auto* error = std::get_if<AuthDatabaseError>(&account)) {
        return *error;
    }
    auto password = Windows1251ToUtf8(std::span<const std::uint8_t>(request.password.data(), request.password.size()));
    if (auto* error = std::get_if<AuthDatabaseError>(&password)) {
        return *error;
    }

    OdbcConnection connection;
    if (auto error = connection.Open(std::get<std::string>(connectionText))) {
        return *error;
    }

    const std::string sql =
        "DECLARE @Result int; EXEC " + std::get<std::string>(quotedProcedure) +
        " @UserID=?, @UserPWDb=?, @UserIP=?, @Result=@Result OUTPUT; SELECT @Result;";
    OdbcStatement statement;
    if (auto error = statement.Prepare(connection, sql)) {
        return *error;
    }

    TextBinding accountBinding{std::get<std::string>(std::move(account))};
    TextBinding passwordBinding{std::get<std::string>(std::move(password))};
    TextBinding ipBinding{LegacyIpv4Text(request.clientIp)};
    if (auto error = BindText(statement.Handle(), 1, accountBinding)) return *error;
    if (auto error = BindText(statement.Handle(), 2, passwordBinding)) return *error;
    if (auto error = BindText(statement.Handle(), 3, ipBinding)) return *error;
    if (auto error = statement.Execute()) return *error;
    if (auto error = AdvanceToFirstRowset(statement.Handle())) return *error;
    if (auto error = statement.FetchOne()) return *error;
    return ReadInt(statement.Handle(), 1);
}

std::variant<AuthExtendedDatabaseResult, AuthDatabaseError>
MssqlOdbcAuthDatabase::AuthenticateExtended(std::string_view procedure,
                                            const AuthDb::AuthQuestData& request)
{
    auto connectionText = BuildConnectionString(m_Impl->authSettings, m_Impl->options);
    if (auto* error = std::get_if<AuthDatabaseError>(&connectionText)) return *error;
    auto quotedProcedure = QuoteProcedure(procedure);
    if (auto* error = std::get_if<AuthDatabaseError>(&quotedProcedure)) return *error;
    auto account = Windows1251ToUtf8(std::span<const std::uint8_t>(request.account.data(), request.account.size()));
    if (auto* error = std::get_if<AuthDatabaseError>(&account)) return *error;

    OdbcConnection connection;
    if (auto error = connection.Open(std::get<std::string>(connectionText))) return *error;

    const std::string sql =
        "DECLARE @suspended datetime, @Assure varbinary(80), @Res int; EXEC " +
        std::get<std::string>(quotedProcedure) +
        " @acc=?, @ip=?, @suspended=@suspended OUTPUT, @Assure=@Assure OUTPUT, "
        "@Res=@Res OUTPUT; SELECT @Res, @Assure, DATEPART(year,@suspended), "
        "DATEPART(month,@suspended), DATEPART(day,@suspended), DATEPART(hour,@suspended), "
        "DATEPART(minute,@suspended), DATEPART(second,@suspended);";
    OdbcStatement statement;
    if (auto error = statement.Prepare(connection, sql)) return *error;

    TextBinding accountBinding{std::get<std::string>(std::move(account))};
    TextBinding ipBinding{LegacyIpv4Text(request.clientIp)};
    if (auto error = BindText(statement.Handle(), 1, accountBinding)) return *error;
    if (auto error = BindText(statement.Handle(), 2, ipBinding)) return *error;
    if (auto error = statement.Execute()) return *error;
    if (auto error = AdvanceToFirstRowset(statement.Handle())) return *error;
    if (auto error = statement.FetchOne()) return *error;

    auto result = ReadInt(statement.Handle(), 1);
    if (auto* error = std::get_if<AuthDatabaseError>(&result)) return *error;
    auto assure = ReadOptionalBinary80(statement.Handle(), 2);
    if (auto* error = std::get_if<AuthDatabaseError>(&assure)) return *error;
    auto suspended = ReadOptionalSuspended(statement.Handle());
    if (auto* error = std::get_if<AuthDatabaseError>(&suspended)) return *error;

    AuthExtendedDatabaseResult output{
        std::get<std::int32_t>(result),
        std::get<std::optional<std::array<std::uint8_t, 80>>>(std::move(assure)),
        std::get<std::optional<AuthDb::LockUntil>>(std::move(suspended)),
    };

    if (output.assure) {
        output.result = 1;
    }
    if (output.result == 1 && !output.assure) {
        return LocalError(std::errc::message_size,
                          "sp_authex вернул result=1 без 80-байтного @Assure");
    }
    if (output.result == -3 && !output.suspended) {
        return LocalError(std::errc::invalid_argument,
                          "sp_authex вернул result=-3 без @suspended");
    }
    return output;
}

std::variant<bool, AuthDatabaseError>
MssqlOdbcAuthDatabase::Lock(std::string_view procedure,
                            const AuthDb::LockQuestData& request)
{
    auto connectionText = BuildConnectionString(m_Impl->authSettings, m_Impl->options);
    if (auto* error = std::get_if<AuthDatabaseError>(&connectionText)) return *error;
    auto quotedProcedure = QuoteProcedure(procedure);
    if (auto* error = std::get_if<AuthDatabaseError>(&quotedProcedure)) return *error;
    auto account = Windows1251ToUtf8(std::span<const std::uint8_t>(request.account.data(), request.account.size()));
    if (auto* error = std::get_if<AuthDatabaseError>(&account)) return *error;

    OdbcConnection connection;
    if (auto error = connection.Open(std::get<std::string>(connectionText))) return *error;

    const std::string sql =
        "DECLARE @Result int; EXEC " + std::get<std::string>(quotedProcedure) +
        " @Account=?, @SuspendTime=?, @Result=@Result OUTPUT; SELECT @Result;";
    OdbcStatement statement;
    if (auto error = statement.Prepare(connection, sql)) return *error;

    TextBinding accountBinding{std::get<std::string>(std::move(account))};
    TimestampBinding timeBinding{ToTimestamp(request.until)};
    if (auto error = BindText(statement.Handle(), 1, accountBinding)) return *error;
    if (auto error = BindTimestamp(statement.Handle(), 2, timeBinding)) return *error;
    if (auto error = statement.Execute()) return *error;
    if (auto error = AdvanceToFirstRowset(statement.Handle())) return *error;
    if (auto error = statement.FetchOne()) return *error;

    auto result = ReadInt(statement.Handle(), 1);
    if (auto* error = std::get_if<AuthDatabaseError>(&result)) return *error;
    return std::get<std::int32_t>(result) == 0;
}

std::optional<AuthDatabaseError>
MssqlOdbcAuthDatabase::WriteServerInfo(std::string_view procedure,
                                       const AuthLocalTime& loggedAt,
                                       std::deque<AuthDb::ServerInfo> entries)
{
    if (entries.empty()) {
        return std::nullopt;
    }

    auto connectionText = BuildConnectionString(m_Impl->logSettings, m_Impl->options);
    if (auto* error = std::get_if<AuthDatabaseError>(&connectionText)) return *error;
    auto quotedProcedure = QuoteProcedure(procedure);
    if (auto* error = std::get_if<AuthDatabaseError>(&quotedProcedure)) return *error;

    OdbcConnection connection;
    if (auto error = connection.Open(std::get<std::string>(connectionText))) return error;

    const std::string sql =
        "EXEC " + std::get<std::string>(quotedProcedure) +
        " @LogTime=?, @ls=?, @ws=?, @gs=?, @Amount=?;";
    OdbcStatement statement;
    if (auto error = statement.Prepare(connection, sql)) return error;

    TimestampBinding timeBinding{ToTimestamp(loggedAt)};
    IntBinding loginBinding{};
    IntBinding worldBinding{};
    IntBinding gameBinding{};
    IntBinding amountBinding{};
    if (auto error = BindTimestamp(statement.Handle(), 1, timeBinding)) return error;
    if (auto error = BindInt(statement.Handle(), 2, loginBinding)) return error;
    if (auto error = BindInt(statement.Handle(), 3, worldBinding)) return error;
    if (auto error = BindInt(statement.Handle(), 4, gameBinding)) return error;
    if (auto error = BindInt(statement.Handle(), 5, amountBinding)) return error;

    for (const AuthDb::ServerInfo& entry : entries) {
        loginBinding.value = static_cast<SQLINTEGER>(entry.loginServerId);
        worldBinding.value = static_cast<SQLINTEGER>(entry.worldServerId);
        gameBinding.value = static_cast<SQLINTEGER>(entry.gameServerId);
        amountBinding.value = static_cast<SQLINTEGER>(entry.playerCount);

        if (auto error = statement.Execute()) return error;
        if (auto error = DrainAllResults(statement.Handle())) return error;
        if (auto error = statement.CloseCursor()) return error;
    }
    return std::nullopt;
}

AuthDatabaseFactory MakeMssqlOdbcDatabaseFactory(MssqlOdbcOptions options)
{
    return [options = std::move(options)]() mutable -> std::unique_ptr<IAuthDatabase> {
        return std::make_unique<MssqlOdbcAuthDatabase>(options);
    };
}
