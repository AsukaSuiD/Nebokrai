#include "playerfillmgr.h"

#include "../../nets/netbilling/message.h"

#include <spdlog/spdlog.h>

#include <algorithm>
#include <chrono>
#include <string>
#include <utility>

namespace Billing
{
namespace
{
constexpr std::int32_t kPlayerFillResponse = 0x000F'F004;
constexpr auto kCadence = std::chrono::seconds(5);

std::string VisibleString(std::span<const std::uint8_t> bytes)
{
    const auto end = std::find(bytes.begin(), bytes.end(), std::uint8_t{});
    return {bytes.begin(), end};
}
}

CPlayerFillMgr::CPlayerFillMgr(ServerCommandHandle sender,
                               RsPlayerFillFactory fillFactory,
                               RsPlayerAccountFactory accountFactory)
    : m_Sender(std::move(sender)),
      m_FillFactory(std::move(fillFactory)),
      m_AccountFactory(std::move(accountFactory))
{
}

CPlayerFillMgr::~CPlayerFillMgr()
{
    RequestStop();
    Join();
}

bool CPlayerFillMgr::Start()
{
    if (m_Worker.joinable() || !m_FillFactory || !m_AccountFactory) return false;
    m_StopRequested.store(false, std::memory_order_release);
    try {
        m_Worker = std::thread(&CPlayerFillMgr::Worker, this);
    } catch (const std::exception& error) {
        spdlog::error("BillingServer: не создан PlayerFill-worker: {}", error.what());
        return false;
    }
    return true;
}

void CPlayerFillMgr::RequestStop() noexcept
{
    m_StopRequested.store(true, std::memory_order_release);
}

void CPlayerFillMgr::Join() noexcept
{
    if (m_Worker.joinable()) m_Worker.join();
}

void CPlayerFillMgr::Worker()
{
    std::unique_ptr<IRsPlayerFill> fillDatabase = m_FillFactory();
    std::unique_ptr<IRsPlayerAccount> accountDatabase = m_AccountFactory();
    if (!fillDatabase || !accountDatabase) {
        spdlog::error("BillingServer: фабрика не создала PlayerFill DB-owner");
        return;
    }

    while (!m_StopRequested.load(std::memory_order_acquire)) {
        const std::vector<PlayerFillInfo> entries = fillDatabase->GetPlayerFillLog();
        for (const PlayerFillInfo& entry : entries) {
            const UserPointOutcome outcome =
                accountDatabase->GetUserPoint(entry.playerAccount);
            BillingNet::CMessage response(kPlayerFillResponse);
            const std::string account = VisibleString(entry.playerAccount);
            response.Base().Add(account.c_str());
            response.Base().Add(outcome.result);
            if (outcome.result == 0) response.Base().Add(outcome.point);
            static_cast<void>(response.SendToAllGS(m_Sender));
        }
        if (!entries.empty()) {
            static_cast<void>(fillDatabase->DeletePlayerFillLog(entries));
        }

        const auto deadline = std::chrono::steady_clock::now() + kCadence;
        while (!m_StopRequested.load(std::memory_order_acquire) &&
               std::chrono::steady_clock::now() < deadline) {
            std::this_thread::sleep_for(std::chrono::milliseconds(50));
        }
    }
}
}
