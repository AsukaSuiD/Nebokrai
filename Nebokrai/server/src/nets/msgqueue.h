#pragma once

#include <cstdint>
#include <deque>
#include <memory>
#include <mutex>
#include <utility>

/*
 * Исходный владелец: nets/msgqueue.cpp / nets/msgqueue.h
 *
 * Точные EXE/PDB всех шести серверов подтверждают один контракт: FIFO,
 * добавление в хвост, извлечение из головы, размер под тем же lock, очистка и
 * атомарная передача всего текущего snapshot. Старые CRITICAL_SECTION,
 * std::deque<CBaseMessage*> и ручное delete являются технической формой.
 *
 * Очередь типизирована, потому что конкретные сетевые владельцы всегда знают свой
 * CMessage. Это сохраняет исходное единственное владение, но не заставляет
 * позднее делать небезопасный downcast из CBaseMessage после PopMessage.
 * eventfd, condition_variable, лимиты и control reserve старого Linux-донора
 * оригинальными EXE/PDB не подтверждены и сюда не переносятся.
 */
template <typename T>
class CMsgQueue
{
public:
    using MessagePtr = std::unique_ptr<T>;
    using MessageQueue = std::deque<MessagePtr>;

    CMsgQueue() = default;
    ~CMsgQueue()
    {
        Clear();
    }

    CMsgQueue(const CMsgQueue&) = delete;
    CMsgQueue& operator=(const CMsgQueue&) = delete;
    CMsgQueue(CMsgQueue&&) = delete;
    CMsgQueue& operator=(CMsgQueue&&) = delete;

    [[nodiscard]] std::int32_t GetSize() const
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionMsgQueue);
        return static_cast<std::int32_t>(static_cast<std::uint32_t>(m_MsgQueue.size()));
    }

    [[nodiscard]] MessagePtr PopMessage()
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionMsgQueue);
        if (m_MsgQueue.empty()) {
            return nullptr;
        }

        MessagePtr message = std::move(m_MsgQueue.front());
        m_MsgQueue.pop_front();
        return message;
    }

    [[nodiscard]] bool PushMessage(MessagePtr message)
    {
        if (!message) {
            return false;
        }

        std::lock_guard<std::mutex> lock(m_CriticalSectionMsgQueue);
        m_MsgQueue.push_back(std::move(message));
        return true;
    }

    [[nodiscard]] MessageQueue GetAllMessage()
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionMsgQueue);
        MessageQueue messages = std::move(m_MsgQueue);
        m_MsgQueue.clear();
        return messages;
    }

    void Clear()
    {
        std::lock_guard<std::mutex> lock(m_CriticalSectionMsgQueue);
        m_MsgQueue.clear();
    }

private:
    mutable std::mutex m_CriticalSectionMsgQueue;
    MessageQueue m_MsgQueue;
};
