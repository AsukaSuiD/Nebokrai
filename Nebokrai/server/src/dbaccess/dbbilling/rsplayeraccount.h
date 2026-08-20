#pragma once

#include "../../billing/appbilling/billingplayermanager.h"
#include "../odbc.h"

#include <cstdint>
#include <deque>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <vector>

namespace Billing
{
struct BillingDatabaseConnection
{
    std::string host;
    std::string database;
    std::string user;
    std::string password;
};

struct BillingDatabaseSettings
{
    BillingDatabaseConnection account;
    BillingDatabaseConnection cashLog;
    Nebokrai::Database::MssqlConnectionOptions transport;
};

struct UserPointOutcome
{
    std::int32_t result{-2};
    std::int32_t point{};
};

struct BuyItemCodeOutcome
{
    std::int32_t result{-5};
    std::vector<std::uint8_t> transactionCode;
    std::int32_t lastPoint{};
    std::optional<TagIncLogNode> incrementLog;
};

struct BuyPlayerItemOutcome
{
    std::int32_t result{-2};
    std::vector<std::uint8_t> transactionCode;
    std::int32_t buyerLastPoint{};
    std::int32_t sellerLastPoint{};
};

class IRsPlayerAccount
{
public:
    virtual ~IRsPlayerAccount() = default;
    [[nodiscard]] virtual UserPointOutcome
    GetUserPoint(std::span<const std::uint8_t> userId) = 0;
    virtual void PutCashLog(const std::deque<TagIncLogNode>& entries) = 0;
    [[nodiscard]] virtual BuyPlayerItemOutcome
    BuyPlayerItem(const TagTradeNode& trade) = 0;
    [[nodiscard]] virtual BuyItemCodeOutcome
    BuyItemCode(const TagTradeNode& trade, bool logServerEnabled) = 0;
};

/*
 * Исходный владелец: dbaccess/dbbilling/rsplayeraccount.cpp / .h.
 * Имена четырёх процедур, параметры, output mapping и fallback-коды взяты из
 * BillingServer EXE/PDB и проверенной Rust-реконструкции. ADO/COM заменён общим
 * ODBC-слоем Nebokrai; credentials никогда не включаются в diagnostics.
 */
class MssqlOdbcRsPlayerAccount final : public IRsPlayerAccount
{
public:
    explicit MssqlOdbcRsPlayerAccount(BillingDatabaseSettings settings);

    [[nodiscard]] UserPointOutcome
    GetUserPoint(std::span<const std::uint8_t> userId) override;
    void PutCashLog(const std::deque<TagIncLogNode>& entries) override;
    [[nodiscard]] BuyPlayerItemOutcome
    BuyPlayerItem(const TagTradeNode& trade) override;
    [[nodiscard]] BuyItemCodeOutcome
    BuyItemCode(const TagTradeNode& trade, bool logServerEnabled) override;

private:
    BillingDatabaseSettings m_Settings;
};

[[nodiscard]] RsPlayerAccountFactory
MakeMssqlOdbcRsPlayerAccountFactory(BillingDatabaseSettings settings);
}
