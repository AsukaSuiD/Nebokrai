#include "msgqueue.h"

#include "basemessage.h"

#include <utility>

CMsgQueue::~CMsgQueue()
{
    Clear();
}

std::int32_t CMsgQueue::GetSize() const
{
    std::lock_guard<std::mutex> lock(m_CriticalSectionMsgQueue);

    // Исходный Windows long был 32-битным; сохраняются младшие 32 бита размера.
    return static_cast<std::int32_t>(static_cast<std::uint32_t>(m_msgQueue.size()));
}

CMsgQueue::MessagePtr CMsgQueue::PopMessage()
{
    std::lock_guard<std::mutex> lock(m_CriticalSectionMsgQueue);
    if (m_msgQueue.empty()) {
        return {};
    }

    MessagePtr message = std::move(m_msgQueue.front());
    m_msgQueue.pop_front();
    return message;
}

bool CMsgQueue::PushMessage(MessagePtr message)
{
    if (!message) {
        return false;
    }

    std::lock_guard<std::mutex> lock(m_CriticalSectionMsgQueue);
    m_msgQueue.push_back(std::move(message));
    return true;
}

CMsgQueue::MessageQueue CMsgQueue::GetAllMessage()
{
    std::lock_guard<std::mutex> lock(m_CriticalSectionMsgQueue);

    // Подтверждённый контракт — один атомарный snapshot: вызывающий получает
    // все текущие сообщения в FIFO-порядке, внутренняя очередь становится пустой.
    MessageQueue messages = std::move(m_msgQueue);
    m_msgQueue.clear();
    return messages;
}

void CMsgQueue::Clear()
{
    std::lock_guard<std::mutex> lock(m_CriticalSectionMsgQueue);

    // unique_ptr уничтожает CBaseMessage под тем же lock, под которым исходный
    // Clear выполнял ручное удаление каждого оставшегося указателя.
    m_msgQueue.clear();
}
