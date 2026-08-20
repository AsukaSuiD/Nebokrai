#pragma once

#include "../../dbaccess/dbbilling/rsplayeraccount.h"
#include "../../dbaccess/dbbilling/rsplayerfillmgr.h"
#include "../../nets/servers.h"

#include <atomic>
#include <memory>
#include <thread>

namespace Billing
{
/*
 * Исходный владелец: appbilling/playerfillmgr.cpp / .h.
 * Один optional worker каждые 5000 мс снимает TOP 50, рассылает 0xFF004 и
 * удаляет весь прочитанный snapshot. Остановка принадлежит общему lifecycle
 * Billing::CGame; отдельного Win32 thread-message здесь не было.
 */
class CPlayerFillMgr
{
public:
    CPlayerFillMgr(ServerCommandHandle sender,
                   RsPlayerFillFactory fillFactory,
                   RsPlayerAccountFactory accountFactory);
    ~CPlayerFillMgr();

    [[nodiscard]] bool Start();
    void RequestStop() noexcept;
    void Join() noexcept;

private:
    void Worker();

    ServerCommandHandle m_Sender;
    RsPlayerFillFactory m_FillFactory;
    RsPlayerAccountFactory m_AccountFactory;
    std::atomic_bool m_StopRequested{false};
    std::thread m_Worker;
};
}
