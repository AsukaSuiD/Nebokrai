#pragma once

#include <array>
#include <cstdint>
#include <span>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/organizingsystem/organizing.cpp/.h.
 * PDB подтверждает значения enum и layout tagMemInfo размером 0xF0; порядок
 * wire-полей подтверждён Rust-реконструкцией. Современные строки и контейнеры
 * используются только снаружи совместимого снимка. Неопределённое старое
 * чтение char-массивов без NUL заменено явной ошибкой кодирования.
 */
enum class EOrganizingOperator : std::int32_t { Delete = 0, Add = 1, Update = 2 };
enum class ECityState : std::int32_t { None = 0, Declare = 1, Mass = 2, Fight = 3 };
enum class EPurviewOwnState : std::int32_t { None = 0, Forbid = 1, Permit = 2 };

struct OrganizingTime {
    std::uint16_t year{}, month{}, dayOfWeek{}, day{};
    std::uint16_t hour{}, minute{}, second{}, milliseconds{};
};

struct OrganizingMemberInfo {
    std::int32_t id{};
    std::array<char, 32> name{};
    std::int32_t level{};
    std::int32_t occupation{};
    std::int32_t jobLevel{};
    std::array<char, 64> title{};
    std::array<EPurviewOwnState, 11> purview{};
    std::array<char, 64> region{};
    OrganizingTime lastOnlineTime{};
    bool contribute{};

    [[nodiscard]] std::string_view Name() const noexcept;
    [[nodiscard]] std::string_view Title() const noexcept;
    [[nodiscard]] std::string_view Region() const noexcept;
    bool SetName(std::string_view value) noexcept;
    bool SetTitle(std::string_view value) noexcept;
    bool SetRegion(std::string_view value) noexcept;
    bool Serialize(std::vector<std::uint8_t>& output) const;
};

static_assert(sizeof(OrganizingTime) == 0x10);
static_assert(sizeof(OrganizingMemberInfo) == 0xF0);
