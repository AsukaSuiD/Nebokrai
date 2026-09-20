#pragma once
#include <atomic>
#include <cstdint>

/* Исходный владелец: WorldServer/appworld/misc.cpp. EXE подтверждает один
 * process-wide счётчик копий, увеличение и суточный сброс в 1. Timer вынесен
 * в process-owner; atomic заменяет старую data race. */
class WorldCopyCounter {
public:
    [[nodiscard]] std::int32_t Current() const noexcept { return m_Value.load(); }
    [[nodiscard]] std::int32_t Next() noexcept { return m_Value.fetch_add(1) + 1; }
    void ResetForNewDay() noexcept { m_Value.store(1); }
private:
    std::atomic<std::int32_t> m_Value{1};
};
