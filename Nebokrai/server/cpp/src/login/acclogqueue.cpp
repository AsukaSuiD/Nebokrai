#include "acclogqueue.h"

#include <utility>

namespace Login
{
void AccLogQueue::Push(AccLogRecord record)
{
    bool released = false;
    {
        std::lock_guard guard(m_Mutex);
        if (m_StopRequested) {
            return;
        }
        m_Logs.push_back(std::move(record));
        if (m_SemaphoreCount < kMaximumSemaphoreCount) {
            ++m_SemaphoreCount;
            released = true;
        }
    }
    if (released) {
        m_NotEmpty.notify_one();
    }
}

std::optional<AccLogRecord> AccLogQueue::Pop()
{
    std::unique_lock lock(m_Mutex);
    m_NotEmpty.wait(lock, [this] {
        return m_StopRequested || m_SemaphoreCount != 0U;
    });
    if (m_StopRequested) {
        return std::nullopt;
    }
    --m_SemaphoreCount;
    if (m_Logs.empty()) {
        return std::nullopt;
    }
    AccLogRecord record = std::move(m_Logs.front());
    m_Logs.pop_front();
    return record;
}

void AccLogQueue::Clear()
{
    std::lock_guard guard(m_Mutex);
    m_Logs.clear();
}

void AccLogQueue::Stop()
{
    {
        std::lock_guard guard(m_Mutex);
        m_StopRequested = true;
        m_Logs.clear();
    }
    m_NotEmpty.notify_all();
}
}
