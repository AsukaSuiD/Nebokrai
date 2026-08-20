#include "playerdataqueue.h"
bool CPlayerDataQueue::Push(PlayerDataJob job){std::lock_guard lock(m_Mutex);if(m_Closed)return false;m_Jobs.push_back(std::move(job));m_Wake.notify_one();return true;}
std::optional<PlayerDataJob>CPlayerDataQueue::TryPop(){std::lock_guard lock(m_Mutex);if(m_Jobs.empty())return{};auto job=std::move(m_Jobs.front());m_Jobs.pop_front();return job;}
std::optional<PlayerDataJob>CPlayerDataQueue::WaitPop(){std::unique_lock lock(m_Mutex);m_Wake.wait(lock,[this]{return m_Closed||!m_Jobs.empty();});if(m_Jobs.empty())return{};auto job=std::move(m_Jobs.front());m_Jobs.pop_front();return job;}
void CPlayerDataQueue::Close(){std::lock_guard lock(m_Mutex);m_Closed=true;m_Wake.notify_all();}
std::size_t CPlayerDataQueue::Size()const{std::lock_guard lock(m_Mutex);return m_Jobs.size();}
