#include "dbqueue.h"

#include <utility>

namespace AuthDb
{
bool ServerInfo::HasSameKey(const ServerInfo& other) const noexcept
{
    return loginServerId == other.loginServerId &&
           worldServerId == other.worldServerId &&
           gameServerId == other.gameServerId;
}

void ServerInfoQueue::PushBack(ServerInfo entry)
{
    std::lock_guard<std::mutex> lock(m_Mutex);
    for (ServerInfo& current : m_Entries) {
        if (current.HasSameKey(entry)) {
            current.playerCount = entry.playerCount;
            return;
        }
    }
    m_Entries.push_back(std::move(entry));
}

std::deque<ServerInfo> ServerInfoQueue::PopAll()
{
    std::lock_guard<std::mutex> lock(m_Mutex);
    std::deque<ServerInfo> entries = std::move(m_Entries);
    m_Entries.clear();
    return entries;
}

std::size_t ServerInfoQueue::Size() const
{
    std::lock_guard<std::mutex> lock(m_Mutex);
    return m_Entries.size();
}
}
