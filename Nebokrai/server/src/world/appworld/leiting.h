#pragma once

#include <cstdint>
#include <optional>

/*
 * Исходный владелец: WorldServer/appworld/leiting.cpp/.h. Rust/EXE подтверждают
 * сравнение только tm_yday и update-kind 1 либо 2 при смене месяца. Обход
 * игроков, сообщения и DB reset выполняет вызывающий process-owner.
 */
struct LeiTingLocalTime { std::int32_t second{}, minute{}, hour{}, monthDay{}, month{}, year{}, weekDay{}, yearDay{}, daylightSaving{}; };
class CLeiTing {
public:
    explicit CLeiTing(LeiTingLocalTime initial = {}) noexcept : m_Last(initial) {}
    [[nodiscard]] std::optional<std::uint32_t> Run(const LeiTingLocalTime& current) noexcept;
    [[nodiscard]] const LeiTingLocalTime& LastUpdate() const noexcept { return m_Last; }
private:
    LeiTingLocalTime m_Last{};
};
