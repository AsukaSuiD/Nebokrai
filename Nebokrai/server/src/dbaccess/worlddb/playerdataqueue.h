#pragma once
#include <condition_variable>
#include <cstdint>
#include <deque>
#include <mutex>
#include <optional>
#include <vector>

/* Исходный владелец: dbaccess/worlddb/playerdataqueue.cpp/.h. Старые critical
 * section/semaphore/list заменены mutex+condition_variable+deque; FIFO,
 * закрытие и пробуждение worker-а сохранены. */
struct PlayerDataJob { std::int32_t playerId{}; std::vector<std::uint8_t> payload; };
class CPlayerDataQueue {
public:
    bool Push(PlayerDataJob job);
    std::optional<PlayerDataJob> TryPop();
    std::optional<PlayerDataJob> WaitPop();
    void Close();
    [[nodiscard]] std::size_t Size() const;
private:
    mutable std::mutex m_Mutex;std::condition_variable m_Wake;std::deque<PlayerDataJob> m_Jobs;bool m_Closed{};
};
