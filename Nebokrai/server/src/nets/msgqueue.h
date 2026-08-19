#pragma once

#include <cstdint>
#include <deque>
#include <memory>
#include <mutex>

class CBaseMessage;

/*
 * Owner: nets/msgqueue.cpp
 *
 * Источник: точные EXE/PDB AuthServer, LoginServer, BillingServer, MiscServer,
 * GameServer и Nworldserver. Во всех шести вариантах подтверждён один контракт:
 * FIFO-очередь CBaseMessage, добавление в хвост, извлечение из головы, размер под
 * тем же lock, очистка оставшихся сообщений владельцем очереди и атомарная
 * передача всего текущего содержимого вызывающему коду.
 *
 * Исходная реализация использовала CRITICAL_SECTION, std::deque<CBaseMessage*>
 * и ручное delete. В новом C++ std::mutex сохраняет критическую секцию, а
 * std::unique_ptr<CBaseMessage> выражает то же единственное владение без ручного
 * освобождения. Добавленные в старом Linux-доноре eventfd, condition_variable,
 * лимиты сообщений/байт и control reserve не подтверждены оригиналом и сюда не
 * переносятся.
 *
 * Старый PushMessage(nullptr) возвращал false; проверенные call sites не
 * использовали результат. Новый API всё равно сохраняет этот отказ, но владение
 * передаётся явно через std::unique_ptr.
 */
class CMsgQueue
{
public:
    using MessagePtr = std::unique_ptr<CBaseMessage>;
    using MessageQueue = std::deque<MessagePtr>;

    CMsgQueue() = default;
    ~CMsgQueue();

    CMsgQueue(const CMsgQueue&) = delete;
    CMsgQueue& operator=(const CMsgQueue&) = delete;

    [[nodiscard]] std::int32_t GetSize() const;
    [[nodiscard]] MessagePtr PopMessage();
    [[nodiscard]] bool PushMessage(MessagePtr message);
    [[nodiscard]] MessageQueue GetAllMessage();
    void Clear();

private:
    mutable std::mutex m_CriticalSectionMsgQueue;
    MessageQueue m_msgQueue;
};
