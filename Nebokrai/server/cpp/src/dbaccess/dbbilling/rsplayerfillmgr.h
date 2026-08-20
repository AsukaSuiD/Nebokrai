#pragma once

#include "rsplayeraccount.h"

#include <cstdint>
#include <functional>
#include <memory>
#include <string>
#include <vector>

namespace Billing
{
struct PlayerFillInfo
{
    std::vector<std::uint8_t> playerAccount;
    std::int32_t id{};
};

class IRsPlayerFill
{
public:
    virtual ~IRsPlayerFill() = default;
    [[nodiscard]] virtual std::vector<PlayerFillInfo> GetPlayerFillLog() = 0;
    [[nodiscard]] virtual std::int32_t
    DeletePlayerFillLog(const std::vector<PlayerFillInfo>& entries) = 0;
};

using RsPlayerFillFactory = std::function<std::unique_ptr<IRsPlayerFill>()>;

/*
 * Исходный владелец: dbaccess/dbbilling/rsplayerfillmgr.cpp / .h.
 * Сохраняются TOP 50, порядок ID, удаление всего снятого snapshot и fallback
 * -2. Recordset ADO заменён общим ODBC-слоем Nebokrai.
 */
class MssqlOdbcRsPlayerFill final : public IRsPlayerFill
{
public:
    MssqlOdbcRsPlayerFill(BillingDatabaseConnection settings,
                         Nebokrai::Database::MssqlConnectionOptions transport = {});

    [[nodiscard]] std::vector<PlayerFillInfo> GetPlayerFillLog() override;
    [[nodiscard]] std::int32_t
    DeletePlayerFillLog(const std::vector<PlayerFillInfo>& entries) override;

private:
    BillingDatabaseConnection m_Settings;
    Nebokrai::Database::MssqlConnectionOptions m_Transport;
};

[[nodiscard]] RsPlayerFillFactory
MakeMssqlOdbcRsPlayerFillFactory(
    BillingDatabaseConnection settings,
    Nebokrai::Database::MssqlConnectionOptions transport = {});
}
