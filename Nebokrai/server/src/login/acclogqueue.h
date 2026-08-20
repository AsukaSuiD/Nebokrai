#pragma once

#include <condition_variable>
#include <cstddef>
#include <deque>
#include <mutex>
#include <string>

/*
 * Исходный владелец: loginserver/acclogqueue.cpp / acclogqueue.h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Компоновка из PDB: Semaphore + Mutex + deque<string>, размер 56.
 * Подтверждённые RVA: pop 0x00401850, конструктор 0x00401A10,
 * деструктор 0x00401A50,
 * clear 0x00401A90, push 0x00401B20.
 *
 * Исходный push под lock добавляет C-string в хвост и ReleaseSemaphore(1).
 * pop сначала бесконечно ждёт semaphore, затем под lock копирует и снимает
 * голову. Win32 semaphore создан с count=0/max=10000; clear чистит deque, но
 * НЕ сбрасывает его count. Поэтому собственный token-count сохраняется отдельно:
 * stale token после clear даёт пустой Pop, как обнулённый char[2048] caller-а,
 * а push сверх 10000 всё ещё кладёт строку в deque, но не создаёт новый token.
 * std::condition_variable + mutex заменяют только Win32 plumbing. Ёмкости deque,
 * устранение дублей или повтор исходный владелец не добавлял.
 */
namespace Login
{
class AccLogQueue
{
public:
    AccLogQueue() = default;

    AccLogQueue(const AccLogQueue&) = delete;
    AccLogQueue& operator=(const AccLogQueue&) = delete;

    void Push(std::string sql);
    [[nodiscard]] std::string Pop();
    void Clear();

private:
    static constexpr std::size_t kMaximumSemaphoreCount = 10000U;

    std::mutex m_Mutex;
    std::condition_variable m_NotEmpty;
    std::deque<std::string> m_Logs;
    std::size_t m_SemaphoreCount{};
};
}
