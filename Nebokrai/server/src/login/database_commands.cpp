#include "database_commands.h"

#include <limits>

namespace login_database {
namespace {

std::string QuoteSqlLiteral(std::string_view value)
{
    return "'" + EscapeSqlLiteral(value) + "'";
}

bool IsTimestamp(std::string_view value)
{
    return IsBoundedValue(value, 19);
}

std::string BuildLatestAccountRowPredicate(std::string_view account)
{
    return " WHERE LogInfoID IN (SELECT TOP 1 LogInfoID FROM logInfo WHERE Account = " +
           QuoteSqlLiteral(account) + " ORDER BY AccountEnterTime DESC)";
}

} // namespace

bool IsBoundedValue(std::string_view value, std::size_t maxBytes, bool allowEmpty)
{
    return (allowEmpty || !value.empty()) && value.size() <= maxBytes;
}

std::string EscapeSqlLiteral(std::string_view value)
{
    std::string escaped;
    escaped.reserve(value.size());
    for (const char character : value) {
        if (character == '\'') {
            escaped.push_back('\'');
        }
        escaped.push_back(character);
    }
    return escaped;
}

std::string BuildAccountEnterLogSql(std::string_view account,
                                    std::string_view timestamp,
                                    std::string_view ip)
{
    if (!IsBoundedValue(account, kCdkeyMaxBytes) || !IsTimestamp(timestamp) ||
        !IsBoundedValue(ip, 16)) {
        return {};
    }

    return "INSERT INTO LogInfo(Account,AccountEnterTime,IP) VALUES(" +
           QuoteSqlLiteral(account) + "," + QuoteSqlLiteral(timestamp) + "," +
           QuoteSqlLiteral(ip) + ")";
}

std::string BuildRoleEnterLogSql(std::string_view account,
                                 std::string_view roleName,
                                 std::string_view timestamp,
                                 std::uint8_t level,
                                 int worldNumber)
{
    if (!IsBoundedValue(account, kCdkeyMaxBytes) ||
        !IsBoundedValue(roleName, kRoleNameMaxBytes) || !IsTimestamp(timestamp) ||
        worldNumber < 0 || worldNumber > std::numeric_limits<std::uint8_t>::max()) {
        return {};
    }

    return "UPDATE LogInfo SET RoleEnterTime=" + QuoteSqlLiteral(timestamp) +
           ", RoleName=" + QuoteSqlLiteral(roleName) +
           ", RoleLevel=" + std::to_string(level) +
           ", WorldNumber=" + std::to_string(worldNumber) +
           BuildLatestAccountRowPredicate(account);
}

std::string BuildLeaveLogSql(std::string_view account,
                             std::string_view timestamp,
                             bool includeRoleLeave)
{
    if (!IsBoundedValue(account, kCdkeyMaxBytes) || !IsTimestamp(timestamp)) {
        return {};
    }

    std::string sql = "UPDATE logInfo SET ";
    if (includeRoleLeave) {
        sql += "RoleLeaveTime=" + QuoteSqlLiteral(timestamp) + ", ";
    }
    sql += "AccountLeaveTime=" + QuoteSqlLiteral(timestamp);
    sql += BuildLatestAccountRowPredicate(account);
    return sql;
}

std::string BuildBillingLoginSql(std::string_view userId,
                                 std::string_view userIp,
                                 int initialResult)
{
    if (!IsBoundedValue(userId, kBillingUserIdMaxBytes) ||
        !IsBoundedValue(userIp, kBillingUserIpMaxBytes)) {
        return {};
    }

    return "DECLARE @Result int = " + std::to_string(initialResult) +
           "; EXEC dbo.getAccInfoEx @UserID=" + QuoteSqlLiteral(userId) +
           ", @UserIP=" + QuoteSqlLiteral(userIp) +
           ", @Result=@Result OUTPUT; SELECT @Result AS Result";
}

std::string BuildServerLogInsertSql(std::uint32_t ip,
                                    int serverType,
                                    int serverId,
                                    std::string_view description)
{
    const std::string ipText = std::to_string(ip & 0xffU) + "." +
                               std::to_string((ip >> 8U) & 0xffU) + "." +
                               std::to_string((ip >> 16U) & 0xffU) + "." +
                               std::to_string((ip >> 24U) & 0xffU);
    return "INSERT INTO dbo.server_log(serv_ip,serv_type,serv_id,description) VALUES(" +
           QuoteSqlLiteral(ipText) + "," + std::to_string(serverType) + "," +
           std::to_string(serverId) + "," + QuoteSqlLiteral(description) + ")";
}

} // namespace login_database
