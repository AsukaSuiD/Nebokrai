#include "servlogqueue.h"

#include <bit>
#include <utility>

namespace Login
{
void ServLogQueue::Push(ServLog record)
{
    std::lock_guard<std::mutex> lock(m_Mutex);
    m_Records.push_back(std::move(record));
}

bool ServLogQueue::Pop(ServLog& record)
{
    std::lock_guard<std::mutex> lock(m_Mutex);
    if (m_Records.empty()) {
        return false;
    }

    record = std::move(m_Records.front());
    m_Records.pop_front();
    return true;
}

std::int32_t ServLogQueue::Size() const
{
    std::lock_guard<std::mutex> lock(m_Mutex);
    const auto legacySize = static_cast<std::uint32_t>(m_Records.size());
    return std::bit_cast<std::int32_t>(legacySize);
}

void ServLogQueue::Clear()
{
    std::lock_guard<std::mutex> lock(m_Mutex);
    m_Records.clear();
}
}
