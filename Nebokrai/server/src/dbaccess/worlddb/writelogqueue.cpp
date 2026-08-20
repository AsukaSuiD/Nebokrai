#include "writelogqueue.h"
bool CWriteLogQueue::Push(std::string line){std::lock_guard lock(m_Mutex);if(m_Closed)return false;m_Lines.push_back(std::move(line));m_Wake.notify_one();return true;}
std::optional<std::string>CWriteLogQueue::WaitPop(){std::unique_lock lock(m_Mutex);m_Wake.wait(lock,[this]{return m_Closed||!m_Lines.empty();});if(m_Lines.empty())return{};auto line=std::move(m_Lines.front());m_Lines.pop_front();return line;}
void CWriteLogQueue::Close(){std::lock_guard lock(m_Mutex);m_Closed=true;m_Wake.notify_all();}
