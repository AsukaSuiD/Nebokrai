#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <span>

/*
 * Исходный владелец: public/guid.cpp / guid.h.
 * Точные пары: Auth, Billing, Login, Misc, Game и World EXE/PDB. Поздняя
 * реконструкция подтверждает общий 16-байтовый формат Microsoft GUID,
 * GUID_INVALID из нулей и служебный GUID_IN_ACTIVE
 * {C02CE7F5-35F4-482D-B927-BBC9893AC6AD}.
 *
 * OpenSSL RAND_bytes заменяет только CoCreateGuid. Версия UUID v4, вариант,
 * смешанный порядок Microsoft GUID, верхний регистр строки и сырой порядок
 * сравнения сохраняются явно. Initialize/Uninitialize оставлены как пустая
 * совместимая граница старого жизненного цикла COM.
 *
 * Повреждённая строка точной длины в исходном IIDFromString могла оставить
 * частично изменённый объект. Наблюдаемое значение не доказано; безопасная
 * реализация оставляет GUID_INVALID и не объявляет частичный разбор контрактом.
 */
class alignas(4) CGUID
{
public:
    constexpr CGUID() noexcept = default;
    explicit CGUID(const char* text) noexcept;

    static void Initialize() noexcept;
    static void Uninitialize() noexcept;
    [[nodiscard]] static bool CreateGUID(CGUID& guid) noexcept;

    [[nodiscard]] bool tostring(char* destination, std::size_t capacity) const noexcept;
    [[nodiscard]] bool IsInvalided() const noexcept;
    [[nodiscard]] constexpr std::span<const std::uint8_t, 16> Bytes() const noexcept
    {
        return m_Bytes;
    }

    [[nodiscard]] friend constexpr bool operator==(const CGUID&, const CGUID&) = default;
    [[nodiscard]] friend constexpr bool operator<(const CGUID& left,
                                                  const CGUID& right) noexcept
    {
        return left.m_Bytes < right.m_Bytes;
    }

    static const CGUID GUID_INVALID;
    static const CGUID GUID_IN_ACTIVE;

private:
    explicit constexpr CGUID(std::array<std::uint8_t, 16> bytes) noexcept
        : m_Bytes(bytes)
    {
    }

    static constexpr CGUID FromBytes(std::array<std::uint8_t, 16> bytes) noexcept
    {
        return CGUID(bytes);
    }

    std::array<std::uint8_t, 16> m_Bytes{};
};

static_assert(sizeof(CGUID) == 16);
static_assert(alignof(CGUID) == 4);

class guid_compare
{
public:
    [[nodiscard]] constexpr bool operator()(const CGUID& left,
                                            const CGUID& right) const noexcept
    {
        return left < right;
    }
};

class hash_guid_compare
{
public:
    [[nodiscard]] std::size_t operator()(const CGUID& value) const noexcept;
    [[nodiscard]] constexpr bool operator()(const CGUID& left,
                                            const CGUID& right) const noexcept
    {
        return left < right;
    }
};

inline const CGUID NULL_GUID = CGUID::GUID_INVALID;
