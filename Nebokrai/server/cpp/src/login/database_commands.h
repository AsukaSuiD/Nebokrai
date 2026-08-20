#pragma once

#include <cstddef>
#include <cstdint>
#include <string>
#include <string_view>

/*
 * Общие SQL-форматы LoginServer, вынесенные из message/runtime-владельцев.
 * Источник контрактов: LoginServer/loginserver.exe + PDB и поздняя Rust-
 * реконструкция. Здесь остаётся только безопасное формирование команд;
 * подтверждённый порядок запросов задаёт вызывающий owner.
 */
namespace login_database {

constexpr std::size_t kCdkeyMaxBytes = 32;
constexpr std::size_t kRoleNameMaxBytes = 20;
constexpr std::size_t kBillingUserIdMaxBytes = 20;
constexpr std::size_t kBillingUserIpMaxBytes = 24;
constexpr std::size_t kServerIpMaxBytes = 15;

[[nodiscard]] bool IsBoundedValue(std::string_view value, std::size_t maxBytes,
                                  bool allowEmpty = false);
[[nodiscard]] std::string EscapeSqlLiteral(std::string_view value);

[[nodiscard]] std::string BuildAccountEnterLogSql(std::string_view account,
                                                   std::string_view timestamp,
                                                   std::string_view ip);
[[nodiscard]] std::string BuildRoleEnterLogSql(std::string_view account,
                                                std::string_view roleName,
                                                std::string_view timestamp,
                                                std::uint8_t level,
                                                int worldNumber);
[[nodiscard]] std::string BuildLeaveLogSql(std::string_view account,
                                            std::string_view timestamp,
                                            bool includeRoleLeave);
[[nodiscard]] std::string BuildBillingLoginSql(std::string_view userId,
                                                std::string_view userIp,
                                                int initialResult);
[[nodiscard]] std::string BuildServerLogInsertSql(std::uint32_t ip,
                                                   int serverType,
                                                   int serverId,
                                                   std::string_view description);

} // namespace login_database
