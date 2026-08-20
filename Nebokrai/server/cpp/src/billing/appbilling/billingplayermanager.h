#pragma once

#include "../../public/guid.h"
#include "../../nets/servers.h"

#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <deque>
#include <functional>
#include <memory>
#include <mutex>
#include <string>
#include <thread>
#include <vector>

namespace Billing
{
class IRsPlayerAccount;

struct TagAccInfo
{
    std::int32_t playerId{};
    std::vector<std::uint8_t> playerIdentity;
    std::int32_t gameServerId{};
};

struct TagTradeNode
{
    std::int32_t tradeType{};
    std::int32_t buyerId{};
    std::int32_t sellerId{};
    std::vector<std::uint8_t> buyerIdentity;
    std::vector<std::uint8_t> sellerIdentity;
    std::vector<std::uint8_t> buyerIp;
    std::vector<std::uint8_t> sellerIp;
    std::vector<std::uint8_t> buyerName;
    std::vector<std::uint8_t> sellerName;
    std::uint32_t yuanbao{};
    std::int32_t goodsId{};
    std::int32_t goodsNumber{};
    std::int32_t gameServerId{};
    std::int32_t sessionId{};
    std::int32_t pluginId{};
    std::int32_t loginServerId{};
    std::int32_t worldServerId{};
    CGUID goodsGuid{CGUID::GUID_INVALID};
};

struct TagIncLogNode
{
    double logTime{};
    std::vector<std::uint8_t> buyerIdentity;
    std::vector<std::uint8_t> buyerIp;
    std::uint32_t yuanbao{};
    std::int32_t goodsId{};
    std::int32_t goodsNumber{};
    std::int32_t loginServerId{};
    std::int32_t worldServerId{};
};

using RsPlayerAccountFactory = std::function<std::unique_ptr<IRsPlayerAccount>()>;

/*
 * Исходный владелец: server/billingserver/appbilling/
 * billingplayermanager.{h,cpp}.
 *
 * BillingServer EXE/PDB и Rust-реконструкция подтверждают три общих FIFO,
 * полный snapshot очередей, порядок AC перед TR, отдельные DB-workers и
 * optional cash-log worker. Здесь они принадлежат одному объекту процесса;
 * Win32 thread messages и static STL заменены std::thread/condition_variable.
 */
class CBillingPlayerManager
{
public:
    CBillingPlayerManager(ServerCommandHandle sender,
                          RsPlayerAccountFactory databaseFactory);
    ~CBillingPlayerManager();

    CBillingPlayerManager(const CBillingPlayerManager&) = delete;
    CBillingPlayerManager& operator=(const CBillingPlayerManager&) = delete;

    [[nodiscard]] bool Start(std::uint32_t databaseWorkers,
                             bool logServerEnabled,
                             std::uint32_t saveLogIntervalMs);
    void RequestStop() noexcept;
    void Join() noexcept;

    [[nodiscard]] bool PushAccountRequest(TagAccInfo request);
    [[nodiscard]] bool PushTradeRequest(TagTradeNode request);
    [[nodiscard]] bool PushIncrementLog(TagIncLogNode record);

private:
    void DatabaseWorker();
    void CashLogWorker();
    [[nodiscard]] bool RunDatabasePass(IRsPlayerAccount& database);
    void RunLogPass(IRsPlayerAccount& database);

    ServerCommandHandle m_Sender;
    RsPlayerAccountFactory m_DatabaseFactory;
    std::mutex m_AccountMutex;
    std::mutex m_TradeMutex;
    std::mutex m_LogMutex;
    std::condition_variable m_WorkReady;
    std::condition_variable m_LogReady;
    std::deque<TagAccInfo> m_AccountRequests;
    std::deque<TagTradeNode> m_TradeRequests;
    std::deque<TagIncLogNode> m_IncrementLogs;
    std::vector<std::thread> m_DatabaseWorkers;
    std::thread m_LogWorker;
    std::atomic_bool m_StopRequested{false};
    std::atomic_bool m_LogServerEnabled{false};
    std::atomic_uint32_t m_SaveLogIntervalMs{0};
};
}
