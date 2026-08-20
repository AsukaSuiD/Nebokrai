#pragma once

#include <condition_variable>
#include <cstddef>
#include <cstdint>
#include <deque>
#include <mutex>
#include <optional>
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
 * std::condition_variable + mutex заменяют только обвязку Win32. Текущий элемент
 * очереди хранит тип и поля account/role/leave-записи отдельно, а SQL формирует
 * единственный AccLogThread. Порядок, token-count, отсутствие дублей и повторов
 * остаются исходными. Stop — только Linux-граница штатного join при Release.
 */
namespace Login
{
enum class AccLogKind
{
    AccountEnter,
    AccountLeave,
    RoleEnter,
    SessionLeave,
};

struct AccLogRecord
{
    AccLogKind kind{AccLogKind::AccountEnter};
    std::string account;
    std::string recordedAt;
    std::string ip;
    std::string roleName;
    std::uint8_t roleLevel{};
    std::int32_t worldNumber{};
};

class AccLogQueue
{
public:
    AccLogQueue() = default;

    AccLogQueue(const AccLogQueue&) = delete;
    AccLogQueue& operator=(const AccLogQueue&) = delete;

    void Push(AccLogRecord record);
    [[nodiscard]] std::optional<AccLogRecord> Pop();
    void Clear();
    void Stop();

private:
    static constexpr std::size_t kMaximumSemaphoreCount = 10000U;

    std::mutex m_Mutex;
    std::condition_variable m_NotEmpty;
    std::deque<AccLogRecord> m_Logs;
    std::size_t m_SemaphoreCount{};
    bool m_StopRequested{};
};
}
