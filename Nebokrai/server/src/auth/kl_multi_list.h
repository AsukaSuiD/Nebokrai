#pragma once

#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <deque>
#include <mutex>
#include <optional>
#include <utility>

/*
 * Исходный владелец: authserver/src/kl_multi_list.h
 *
 * Точная пара: AuthServer/authserver.exe + AuthServer/authserver.pdb.
 * Исходный путь владельца в PDB:
 * Путь владельца в PDB: h:\fengyun\fy_russia\src\server\authserver\src\kl_multi_list.h
 * Подтверждённые RVA для db_element_type: размер 0x2CB0, конструктор 0x46A0,
 * деструктор 0x4710, pop_front 0x4750, push_back 0x5B10; для ServerInfo:
 * конструктор 0x5220, push_back 0x16C40, pop_all 0x191D0.
 *
 * Доказанный контракт — FIFO под одним lock. Wait-pop спит на пустой очереди;
 * push будит одного ожидающего только при переходе empty -> non-empty. Size —
 * отдельный snapshot: DB worker сначала читает размер, затем выполняет именно
 * столько отдельных ожидающих pop. Эта межпоточная семантика сохранена.
 *
 * Windows critical section/semaphore/condition заменены стандартными
 * std::mutex/std::condition_variable. Stop расширяет только пустое ожидание и
 * нужен для owned Linux worker shutdown; уже доступный элемент передаётся до
 * проверки stop, как в поздней доказанной реконструкции.
 */
template <typename T>
class MultiList
{
public:
    [[nodiscard]] std::uint32_t Size() const
    {
        std::lock_guard<std::mutex> lock(m_Mutex);
        return static_cast<std::uint32_t>(m_Elements.size());
    }

    void PushBack(T element)
    {
        std::lock_guard<std::mutex> lock(m_Mutex);
        const bool wasEmpty = m_Elements.empty();
        m_Elements.push_back(std::move(element));
        if (wasEmpty) {
            m_Available.notify_one();
        }
    }

    [[nodiscard]] T PopFrontWait()
    {
        std::unique_lock<std::mutex> lock(m_Mutex);
        m_Available.wait(lock, [this] { return !m_Elements.empty(); });
        T element = std::move(m_Elements.front());
        m_Elements.pop_front();
        return element;
    }

    [[nodiscard]] std::optional<T>
    PopFrontWaitUntilStopped(const std::atomic_bool& stopped)
    {
        std::unique_lock<std::mutex> lock(m_Mutex);
        while (m_Elements.empty()) {
            if (stopped.load(std::memory_order_acquire)) {
                return std::nullopt;
            }
            m_Available.wait(lock);
        }
        T element = std::move(m_Elements.front());
        m_Elements.pop_front();
        return element;
    }

    [[nodiscard]] std::optional<T> TryPopFront()
    {
        std::lock_guard<std::mutex> lock(m_Mutex);
        if (m_Elements.empty()) {
            return std::nullopt;
        }
        T element = std::move(m_Elements.front());
        m_Elements.pop_front();
        return element;
    }

    void WakeAll()
    {
        std::lock_guard<std::mutex> lock(m_Mutex);
        m_Available.notify_all();
    }

private:
    mutable std::mutex m_Mutex;
    std::condition_variable m_Available;
    std::deque<T> m_Elements;
};
