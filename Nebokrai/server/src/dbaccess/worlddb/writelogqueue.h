#pragma once
#include <condition_variable>
#include <deque>
#include <mutex>
#include <optional>
#include <string>

/* Исходный владелец: dbaccess/worlddb/writelogqueue.cpp/.h. Владеющая FIFO
 * очередь заменяет raw char* и Windows semaphore; строки не теряют байты. */
class CWriteLogQueue {
public:
    bool Push(std::string line);std::optional<std::string> WaitPop();void Close();
private:std::mutex m_Mutex;std::condition_variable m_Wake;std::deque<std::string> m_Lines;bool m_Closed{};
};
