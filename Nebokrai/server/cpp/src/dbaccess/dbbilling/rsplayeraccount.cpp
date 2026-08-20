#include "rsplayeraccount.h"

#include <spdlog/spdlog.h>

#include <algorithm>
#include <array>
#include <bit>
#include <chrono>
#include <cmath>
#include <cstring>
#include <ctime>
#include <limits>
#include <utility>

namespace Billing
{
namespace
{
using Nebokrai::Database::OdbcConnection;
using Nebokrai::Database::OdbcError;
using Nebokrai::Database::OdbcStatement;

std::string VisibleString(std::span<const std::uint8_t> bytes)
{
    const auto end = std::find(bytes.begin(), bytes.end(), std::uint8_t{});
    return {bytes.begin(), end};
}

void LogDatabaseError(std::string_view operation, const OdbcError& error)
{
    spdlog::error("BillingServer: ошибка БД {}: {}", operation, error.detail);
}

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

bool BindText(OdbcStatement& statement,
              SQLUSMALLINT index,
              std::string& value,
              SQLLEN& length,
              SQLULEN maximum)
{
    length = static_cast<SQLLEN>(value.size());
    return Nebokrai::Database::OdbcSucceeded(SQLBindParameter(
        statement.NativeHandle(), index, SQL_PARAM_INPUT, SQL_C_CHAR, SQL_VARCHAR,
        maximum, 0, value.data(), static_cast<SQLLEN>(value.size() + 1U), &length));
}

bool BindI32(OdbcStatement& statement,
             SQLUSMALLINT index,
             SQLINTEGER& value,
             SQLLEN& indicator)
{
    indicator = 0;
    return Nebokrai::Database::OdbcSucceeded(SQLBindParameter(
        statement.NativeHandle(), index, SQL_PARAM_INPUT, SQL_C_SLONG, SQL_INTEGER,
        0, 0, &value, 0, &indicator));
}

bool BindI64(OdbcStatement& statement,
             SQLUSMALLINT index,
             SQLBIGINT& value,
             SQLLEN& indicator)
{
    indicator = 0;
    return Nebokrai::Database::OdbcSucceeded(SQLBindParameter(
        statement.NativeHandle(), index, SQL_PARAM_INPUT, SQL_C_SBIGINT, SQL_BIGINT,
        0, 0, &value, 0, &indicator));
}

bool BindTimestamp(OdbcStatement& statement,
                   SQLUSMALLINT index,
                   TIMESTAMP_STRUCT& value,
                   SQLLEN& indicator)
{
    indicator = 0;
    return Nebokrai::Database::OdbcSucceeded(SQLBindParameter(
        statement.NativeHandle(), index, SQL_PARAM_INPUT, SQL_C_TYPE_TIMESTAMP,
        SQL_TYPE_TIMESTAMP, 23, 3, &value, sizeof(value), &indicator));
}

std::optional<OdbcError> FetchResultRow(OdbcStatement& statement,
                                        std::string_view operation)
{
    while (true) {
        SQLSMALLINT columns = 0;
        const SQLRETURN columnResult = SQLNumResultCols(statement.NativeHandle(), &columns);
        if (!Nebokrai::Database::OdbcSucceeded(columnResult)) {
            return Nebokrai::Database::OdbcDiagnostic(
                SQL_HANDLE_STMT, statement.NativeHandle(), operation);
        }
        if (columns > 0) {
            return statement.FetchOne();
        }
        const SQLRETURN next = SQLMoreResults(statement.NativeHandle());
        if (next == SQL_NO_DATA) {
            return Nebokrai::Database::OdbcLocalError(
                std::errc::no_message_available,
                std::string(operation) + ": отсутствует результирующий SELECT");
        }
        if (!Nebokrai::Database::OdbcSucceeded(next)) {
            return Nebokrai::Database::OdbcDiagnostic(
                SQL_HANDLE_STMT, statement.NativeHandle(), operation);
        }
    }
}

bool ReadI32(OdbcStatement& statement, SQLUSMALLINT column, std::int32_t& value)
{
    SQLINTEGER output = 0;
    SQLLEN indicator = 0;
    const SQLRETURN result = SQLGetData(statement.NativeHandle(), column, SQL_C_SLONG,
                                        &output, sizeof(output), &indicator);
    if (!Nebokrai::Database::OdbcSucceeded(result) || indicator == SQL_NULL_DATA) {
        return false;
    }
    value = static_cast<std::int32_t>(output);
    return true;
}

bool ReadBytes(OdbcStatement& statement,
               SQLUSMALLINT column,
               std::vector<std::uint8_t>& value)
{
    std::array<std::uint8_t, 500> buffer{};
    SQLLEN indicator = 0;
    const SQLRETURN result = SQLGetData(statement.NativeHandle(), column, SQL_C_BINARY,
                                        buffer.data(), buffer.size(), &indicator);
    if (!Nebokrai::Database::OdbcSucceeded(result) || indicator == SQL_NULL_DATA) {
        return false;
    }
    const std::size_t size = std::min<std::size_t>(
        indicator < 0 ? 0U : static_cast<std::size_t>(indicator), buffer.size());
    const auto end = std::find(buffer.begin(), buffer.begin() + size, std::uint8_t{});
    value.assign(buffer.begin(), end);
    return true;
}

TIMESTAMP_STRUCT OleTimestamp(double value)
{
    constexpr double kOleDaysAtUnixEpoch = 25'569.0;
    const auto seconds = static_cast<std::int64_t>(
        std::floor((value - kOleDaysAtUnixEpoch) * 86'400.0));
    const std::time_t raw = static_cast<std::time_t>(seconds);
    std::tm broken{};
#if defined(_WIN32)
    gmtime_s(&broken, &raw);
#else
    gmtime_r(&raw, &broken);
#endif
    TIMESTAMP_STRUCT timestamp{};
    timestamp.year = static_cast<SQLSMALLINT>(broken.tm_year + 1900);
    timestamp.month = static_cast<SQLUSMALLINT>(broken.tm_mon + 1);
    timestamp.day = static_cast<SQLUSMALLINT>(broken.tm_mday);
    timestamp.hour = static_cast<SQLUSMALLINT>(broken.tm_hour);
    timestamp.minute = static_cast<SQLUSMALLINT>(broken.tm_min);
    timestamp.second = static_cast<SQLUSMALLINT>(broken.tm_sec);
    return timestamp;
}

double CurrentOleDate()
{
    constexpr double kOleDaysAtUnixEpoch = 25'569.0;
    const auto now = std::chrono::system_clock::now().time_since_epoch();
    return kOleDaysAtUnixEpoch +
           std::chrono::duration<double>(now).count() / 86'400.0;
}
}

MssqlOdbcRsPlayerAccount::MssqlOdbcRsPlayerAccount(BillingDatabaseSettings settings)
    : m_Settings(std::move(settings))
{
}

UserPointOutcome MssqlOdbcRsPlayerAccount::GetUserPoint(
    std::span<const std::uint8_t> userId)
{
    UserPointOutcome outcome;
    OdbcConnection connection;
    if (auto error = OpenDatabase(m_Settings.account, m_Settings.transport, connection)) {
        LogDatabaseError("GetUserPoint/connect", *error);
        return outcome;
    }
    OdbcStatement statement;
    if (auto error = statement.Create(connection)) {
        LogDatabaseError("GetUserPoint/create", *error);
        return outcome;
    }
    constexpr std::string_view sql =
        "SET NOCOUNT ON; DECLARE @Point int, @Result int; "
        "EXEC [GetUserPoint] @UserID=CONVERT(varchar(200), ?), "
        "@Point=@Point OUTPUT, @Result=@Result OUTPUT; SELECT @Result, @Point;";
    if (auto error = statement.Prepare(sql)) {
        LogDatabaseError("GetUserPoint/prepare", *error);
        return outcome;
    }
    std::string identity = VisibleString(userId);
    SQLLEN identityLength = 0;
    if (!BindText(statement, 1, identity, identityLength, 200)) {
        LogDatabaseError("GetUserPoint/bind",
                         Nebokrai::Database::OdbcDiagnostic(
                             SQL_HANDLE_STMT, statement.NativeHandle(), "SQLBindParameter"));
        return outcome;
    }
    if (auto error = statement.Execute()) {
        LogDatabaseError("GetUserPoint/execute", *error);
        return outcome;
    }
    if (auto error = FetchResultRow(statement, "GetUserPoint/fetch")) {
        LogDatabaseError("GetUserPoint/fetch", *error);
        return outcome;
    }
    if (!ReadI32(statement, 1, outcome.result) || !ReadI32(statement, 2, outcome.point)) {
        spdlog::error("BillingServer: GetUserPoint не вернул обязательные output-поля");
        return UserPointOutcome{};
    }
    return outcome;
}

void MssqlOdbcRsPlayerAccount::PutCashLog(const std::deque<TagIncLogNode>& entries)
{
    if (entries.empty()) {
        return;
    }
    OdbcConnection connection;
    if (auto error = OpenDatabase(m_Settings.cashLog, m_Settings.transport, connection)) {
        LogDatabaseError("PutCashLog/connect", *error);
        return;
    }
    constexpr std::string_view sql =
        "EXEC [PutCashLog] @LogTime=?, @UserAcc=CONVERT(varchar(200), ?), "
        "@UserIp=CONVERT(varchar(200), ?), @Money=CONVERT(int, ?), "
        "@ItemIdx=?, @itemNum=?, @ls=?, @ws=?;";
    for (const TagIncLogNode& entry : entries) {
        OdbcStatement statement;
        if (auto error = statement.Create(connection)) {
            LogDatabaseError("PutCashLog/create", *error);
            return;
        }
        if (auto error = statement.Prepare(sql)) {
            LogDatabaseError("PutCashLog/prepare", *error);
            return;
        }
        TIMESTAMP_STRUCT timestamp = OleTimestamp(entry.logTime);
        std::string account = VisibleString(entry.buyerIdentity);
        std::string address = VisibleString(entry.buyerIp);
        SQLBIGINT amount = static_cast<SQLBIGINT>(entry.yuanbao);
        SQLINTEGER integers[] = {entry.goodsId, entry.goodsNumber,
                                 entry.loginServerId, entry.worldServerId};
        SQLLEN indicators[8]{};
        const bool bound = BindTimestamp(statement, 1, timestamp, indicators[0]) &&
            BindText(statement, 2, account, indicators[1], 200) &&
            BindText(statement, 3, address, indicators[2], 200) &&
            BindI64(statement, 4, amount, indicators[3]) &&
            BindI32(statement, 5, integers[0], indicators[4]) &&
            BindI32(statement, 6, integers[1], indicators[5]) &&
            BindI32(statement, 7, integers[2], indicators[6]) &&
            BindI32(statement, 8, integers[3], indicators[7]);
        if (!bound) {
            LogDatabaseError("PutCashLog/bind",
                             Nebokrai::Database::OdbcDiagnostic(
                                 SQL_HANDLE_STMT, statement.NativeHandle(), "SQLBindParameter"));
            return;
        }
        if (auto error = statement.Execute()) {
            LogDatabaseError("PutCashLog/execute", *error);
            return;
        }
    }
}

BuyPlayerItemOutcome MssqlOdbcRsPlayerAccount::BuyPlayerItem(const TagTradeNode& trade)
{
    BuyPlayerItemOutcome outcome;
    OdbcConnection connection;
    if (auto error = OpenDatabase(m_Settings.account, m_Settings.transport, connection)) {
        LogDatabaseError("buyPlayerItem/connect", *error);
        return outcome;
    }
    OdbcStatement statement;
    if (auto error = statement.Create(connection)) {
        LogDatabaseError("buyPlayerItem/create", *error);
        return outcome;
    }
    constexpr std::string_view sql =
        "SET NOCOUNT ON; DECLARE @Result int, @TranCode varchar(500), "
        "@UIDfromLastPoint int, @UIDtoLastPoint int; EXEC [buyPlayerItem] "
        "@UIDfrom=CONVERT(varchar(200), ?), @Ipfrom=CONVERT(varchar(200), ?), "
        "@Cnamefrom=CONVERT(varchar(200), ?), @UIDto=CONVERT(varchar(200), ?), "
        "@Ipto=CONVERT(varchar(200), ?), @Cnameto=CONVERT(varchar(200), ?), "
        "@WorldId=?, @ItemIdx=?, @ItemNum=?, @Amount=CONVERT(int, ?), "
        "@Result=@Result OUTPUT, @TranCode=@TranCode OUTPUT, "
        "@UIDfromLastPoint=@UIDfromLastPoint OUTPUT, "
        "@UIDtoLastPoint=@UIDtoLastPoint OUTPUT; "
        "SELECT @Result, CONVERT(varbinary(500), @TranCode), "
        "@UIDfromLastPoint, @UIDtoLastPoint;";
    if (auto error = statement.Prepare(sql)) {
        LogDatabaseError("buyPlayerItem/prepare", *error);
        return outcome;
    }
    std::string text[] = {VisibleString(trade.sellerIdentity),
                          VisibleString(trade.sellerIp),
                          VisibleString(trade.sellerName),
                          VisibleString(trade.buyerIdentity),
                          VisibleString(trade.buyerIp),
                          VisibleString(trade.buyerName)};
    SQLINTEGER integers[] = {trade.worldServerId, trade.goodsId, trade.goodsNumber};
    SQLBIGINT amount = static_cast<SQLBIGINT>(trade.yuanbao);
    SQLLEN indicators[10]{};
    bool bound = true;
    for (SQLUSMALLINT index = 0; index < 6; ++index) {
        bound = bound && BindText(statement, index + 1, text[index], indicators[index], 200);
    }
    bound = bound && BindI32(statement, 7, integers[0], indicators[6]) &&
        BindI32(statement, 8, integers[1], indicators[7]) &&
        BindI32(statement, 9, integers[2], indicators[8]) &&
        BindI64(statement, 10, amount, indicators[9]);
    if (!bound) {
        LogDatabaseError("buyPlayerItem/bind",
                         Nebokrai::Database::OdbcDiagnostic(
                             SQL_HANDLE_STMT, statement.NativeHandle(), "SQLBindParameter"));
        return outcome;
    }
    if (auto error = statement.Execute()) {
        LogDatabaseError("buyPlayerItem/execute", *error);
        return outcome;
    }
    if (auto error = FetchResultRow(statement, "buyPlayerItem/fetch")) {
        LogDatabaseError("buyPlayerItem/fetch", *error);
        return outcome;
    }
    if (!ReadI32(statement, 1, outcome.result) ||
        !ReadBytes(statement, 2, outcome.transactionCode) ||
        !ReadI32(statement, 3, outcome.sellerLastPoint) ||
        !ReadI32(statement, 4, outcome.buyerLastPoint)) {
        spdlog::error("BillingServer: buyPlayerItem не вернул обязательные output-поля");
        return BuyPlayerItemOutcome{};
    }
    return outcome;
}

BuyItemCodeOutcome MssqlOdbcRsPlayerAccount::BuyItemCode(const TagTradeNode& trade,
                                                         bool logServerEnabled)
{
    BuyItemCodeOutcome outcome;
    OdbcConnection connection;
    if (auto error = OpenDatabase(m_Settings.account, m_Settings.transport, connection)) {
        LogDatabaseError("buyItemCode/connect", *error);
        return outcome;
    }
    OdbcStatement statement;
    if (auto error = statement.Create(connection)) {
        LogDatabaseError("buyItemCode/create", *error);
        return outcome;
    }
    constexpr std::string_view sql =
        "SET NOCOUNT ON; DECLARE @Result int, @TranCode varchar(500), "
        "@LastPoint int; EXEC [buyItemCode] "
        "@UserID=CONVERT(varchar(32), ?), @ItemIndex=?, @Qty=?, "
        "@Amount=CONVERT(int, ?), @IP=CONVERT(varchar(200), ?), "
        "@World=CONVERT(varchar(200), ?), "
        "@Character_Name=CONVERT(varchar(200), ?), @Result=@Result OUTPUT, "
        "@TranCode=@TranCode OUTPUT, @LastPoint=@LastPoint OUTPUT; "
        "SELECT @Result, CONVERT(varbinary(500), @TranCode), @LastPoint;";
    if (auto error = statement.Prepare(sql)) {
        LogDatabaseError("buyItemCode/prepare", *error);
        return outcome;
    }
    std::string identity = VisibleString(trade.buyerIdentity);
    std::string address = VisibleString(trade.buyerIp);
    std::string world = std::to_string(trade.worldServerId);
    std::string character = VisibleString(trade.buyerName);
    SQLINTEGER goodsId = trade.goodsId;
    SQLINTEGER quantity = trade.goodsNumber;
    SQLBIGINT amount = static_cast<SQLBIGINT>(trade.yuanbao);
    SQLLEN indicators[7]{};
    const bool bound = BindText(statement, 1, identity, indicators[0], 32) &&
        BindI32(statement, 2, goodsId, indicators[1]) &&
        BindI32(statement, 3, quantity, indicators[2]) &&
        BindI64(statement, 4, amount, indicators[3]) &&
        BindText(statement, 5, address, indicators[4], 200) &&
        BindText(statement, 6, world, indicators[5], 200) &&
        BindText(statement, 7, character, indicators[6], 200);
    if (!bound) {
        LogDatabaseError("buyItemCode/bind",
                         Nebokrai::Database::OdbcDiagnostic(
                             SQL_HANDLE_STMT, statement.NativeHandle(), "SQLBindParameter"));
        return outcome;
    }
    if (auto error = statement.Execute()) {
        LogDatabaseError("buyItemCode/execute", *error);
        return outcome;
    }
    if (auto error = FetchResultRow(statement, "buyItemCode/fetch")) {
        LogDatabaseError("buyItemCode/fetch", *error);
        return outcome;
    }
    if (!ReadI32(statement, 1, outcome.result) ||
        !ReadBytes(statement, 2, outcome.transactionCode) ||
        !ReadI32(statement, 3, outcome.lastPoint)) {
        spdlog::error("BillingServer: buyItemCode не вернул обязательные output-поля");
        return BuyItemCodeOutcome{};
    }
    if (outcome.result == 0 && logServerEnabled) {
        outcome.incrementLog = TagIncLogNode{
            .logTime = CurrentOleDate(),
            .buyerIdentity = trade.buyerIdentity,
            .buyerIp = trade.buyerIp,
            .yuanbao = trade.yuanbao,
            .goodsId = trade.goodsId,
            .goodsNumber = trade.goodsNumber,
            .loginServerId = trade.loginServerId,
            .worldServerId = trade.worldServerId};
    }
    return outcome;
}

RsPlayerAccountFactory
MakeMssqlOdbcRsPlayerAccountFactory(BillingDatabaseSettings settings)
{
    return [settings = std::move(settings)] {
        return std::make_unique<MssqlOdbcRsPlayerAccount>(settings);
    };
}
}
