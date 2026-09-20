#include "billingplayermanager.h"

#include "../../dbaccess/dbbilling/rsplayeraccount.h"
#include "../../nets/netbilling/message.h"
#include "../../public/readwrite.h"

#include <spdlog/spdlog.h>

#include <chrono>
#include <algorithm>
#include <cstring>
#include <sstream>
#include <utility>

namespace Billing
{
namespace
{
constexpr std::int32_t kAccountResponse = 0x000F'F001;
constexpr std::int32_t kIncrementPurchaseResponse = 0x000F'F002;
constexpr std::int32_t kPlayerTradeResponse = 0x000F'F003;
constexpr std::int32_t kMaximumPurchaseGoodsNumber = 1000;
constexpr auto kDatabaseCadence = std::chrono::milliseconds(1);

std::string TransactionCode(const std::vector<std::uint8_t>& bytes)
{
    const auto end = std::find(bytes.begin(), bytes.end(), std::uint8_t{});
    return {bytes.begin(), end};
}
}

CBillingPlayerManager::CBillingPlayerManager(ServerCommandHandle sender,
                                             RsPlayerAccountFactory databaseFactory)
    : m_Sender(std::move(sender)),
      m_DatabaseFactory(std::move(databaseFactory))
{
}

CBillingPlayerManager::~CBillingPlayerManager()
{
    RequestStop();
    Join();
}

bool CBillingPlayerManager::Start(std::uint32_t databaseWorkers,
                                  bool logServerEnabled,
                                  std::uint32_t saveLogIntervalMs)
{
    if (!m_DatabaseWorkers.empty() || m_LogWorker.joinable() ||
        !m_DatabaseFactory || databaseWorkers == 0) {
        return false;
    }

    m_StopRequested.store(false, std::memory_order_release);
    m_LogServerEnabled.store(logServerEnabled, std::memory_order_release);
    m_SaveLogIntervalMs.store(saveLogIntervalMs, std::memory_order_release);
    try {
        m_DatabaseWorkers.reserve(databaseWorkers);
        for (std::uint32_t index = 0; index < databaseWorkers; ++index) {
            m_DatabaseWorkers.emplace_back(&CBillingPlayerManager::DatabaseWorker, this);
        }
        if (logServerEnabled) {
            // Ошибка optional cash-log thread не делала Start фатальным.
            try {
                m_LogWorker = std::thread(&CBillingPlayerManager::CashLogWorker, this);
            } catch (const std::exception& error) {
                spdlog::error("BillingServer: не создан обработчик CashLog: {}", error.what());
            }
        }
    } catch (const std::exception& error) {
        spdlog::error("BillingServer: не созданы обработчики БД: {}", error.what());
        RequestStop();
        Join();
        return false;
    }
    return true;
}

void CBillingPlayerManager::RequestStop() noexcept
{
    m_StopRequested.store(true, std::memory_order_release);
    m_WorkReady.notify_all();
    m_LogReady.notify_all();
}

void CBillingPlayerManager::Join() noexcept
{
    for (std::thread& worker : m_DatabaseWorkers) {
        if (worker.joinable()) {
            worker.join();
        }
    }
    m_DatabaseWorkers.clear();
    if (m_LogWorker.joinable()) {
        m_LogWorker.join();
    }
}

bool CBillingPlayerManager::PushAccountRequest(TagAccInfo request)
{
    {
        std::lock_guard<std::mutex> lock(m_AccountMutex);
        m_AccountRequests.push_back(std::move(request));
    }
    m_WorkReady.notify_one();
    return true;
}

bool CBillingPlayerManager::PushTradeRequest(TagTradeNode request)
{
    {
        std::lock_guard<std::mutex> lock(m_TradeMutex);
        m_TradeRequests.push_back(std::move(request));
    }
    m_WorkReady.notify_one();
    return true;
}

bool CBillingPlayerManager::PushIncrementLog(TagIncLogNode record)
{
    {
        std::lock_guard<std::mutex> lock(m_LogMutex);
        m_IncrementLogs.push_back(std::move(record));
    }
    m_LogReady.notify_one();
    return true;
}

void CBillingPlayerManager::DatabaseWorker()
{
    std::unique_ptr<IRsPlayerAccount> database = m_DatabaseFactory();
    if (!database) {
        spdlog::error("BillingServer: фабрика БД не создала worker-owner");
        return;
    }

    while (!m_StopRequested.load(std::memory_order_acquire)) {
        if (!RunDatabasePass(*database)) {
            break;
        }
        std::unique_lock<std::mutex> lock(m_AccountMutex);
        m_WorkReady.wait_for(lock, kDatabaseCadence, [this] {
            return m_StopRequested.load(std::memory_order_acquire) ||
                   !m_AccountRequests.empty();
        });
    }
}

void CBillingPlayerManager::CashLogWorker()
{
    std::unique_ptr<IRsPlayerAccount> database = m_DatabaseFactory();
    if (!database) {
        spdlog::error("BillingServer: фабрика БД не создала CashLog-owner");
        return;
    }

    do {
        RunLogPass(*database);
        const auto interval =
            std::chrono::milliseconds(m_SaveLogIntervalMs.load(std::memory_order_acquire));
        std::unique_lock<std::mutex> lock(m_LogMutex);
        m_LogReady.wait_for(lock, interval, [this] {
            return m_StopRequested.load(std::memory_order_acquire);
        });
    } while (!m_StopRequested.load(std::memory_order_acquire));
}

bool CBillingPlayerManager::RunDatabasePass(IRsPlayerAccount& database)
{
    std::deque<TagAccInfo> accounts;
    {
        std::lock_guard<std::mutex> lock(m_AccountMutex);
        accounts.swap(m_AccountRequests);
    }
    for (const TagAccInfo& request : accounts) {
        const UserPointOutcome outcome = database.GetUserPoint(request.playerIdentity);
        BillingNet::CMessage response(kAccountResponse);
        response.Base().Add(request.playerId);
        response.Base().Add(outcome.result);
        if (outcome.result == 0) {
            response.Base().Add(outcome.point);
        }
        static_cast<void>(response.SendToGS(m_Sender, request.gameServerId));
    }

    std::deque<TagTradeNode> trades;
    {
        std::lock_guard<std::mutex> lock(m_TradeMutex);
        trades.swap(m_TradeRequests);
    }
    while (!trades.empty()) {
        TagTradeNode trade = std::move(trades.front());
        trades.pop_front();

        if (trade.sellerIdentity.empty()) {
            BuyItemCodeOutcome outcome = database.BuyItemCode(
                trade, m_LogServerEnabled.load(std::memory_order_acquire));
            if (outcome.incrementLog) {
                static_cast<void>(PushIncrementLog(std::move(*outcome.incrementLog)));
            }

            // Исходник проверял лимит после уже выполненной процедуры БД.
            if (trade.goodsNumber > kMaximumPurchaseGoodsNumber) {
                std::ostringstream line;
                line << "playerid:[" << trade.buyerId << "] buy goods:["
                     << trade.goodsId << "];goodsnumber:[" << trade.goodsNumber << ']';
                PutStringToFile("Increment_error_log", line.str());
                return false;
            }

            BillingNet::CMessage response(kIncrementPurchaseResponse);
            response.Base().Add(trade.buyerId);
            response.Base().Add(outcome.result);
            if (outcome.result == 0) {
                response.Base().Add(outcome.lastPoint);
                response.Base().Add(trade.yuanbao);
                response.Base().Add(trade.goodsId);
                response.Base().Add(trade.goodsNumber);
                response.Base().Add(trade.sellerId);
                const std::string code = TransactionCode(outcome.transactionCode);
                response.Base().Add(code.c_str());
            }
            static_cast<void>(response.SendToGS(m_Sender, trade.gameServerId));
            continue;
        }

        const BuyPlayerItemOutcome outcome = database.BuyPlayerItem(trade);
        BillingNet::CMessage response(kPlayerTradeResponse);
        response.Base().Add(trade.buyerId);
        response.Base().Add(trade.sellerId);
        response.Base().Add(outcome.result);
        if (outcome.result == 0) {
            response.Base().Add(outcome.buyerLastPoint);
            response.Base().Add(outcome.sellerLastPoint);
            response.Base().Add(trade.tradeType);
            response.Base().Add(trade.sessionId);
            response.Base().Add(trade.pluginId);
            response.Base().Add(trade.yuanbao);
            response.Base().Add(trade.goodsGuid);
            const std::string code = TransactionCode(outcome.transactionCode);
            response.Base().Add(code.c_str());
        }
        static_cast<void>(response.SendToGS(m_Sender, trade.gameServerId));
    }
    return true;
}

void CBillingPlayerManager::RunLogPass(IRsPlayerAccount& database)
{
    std::deque<TagIncLogNode> logs;
    {
        std::lock_guard<std::mutex> lock(m_LogMutex);
        logs.swap(m_IncrementLogs);
    }
    if (!logs.empty()) {
        database.PutCashLog(logs);
    }
}
}
