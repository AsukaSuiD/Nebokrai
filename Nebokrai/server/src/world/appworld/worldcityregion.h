#pragma once

#include "worldwarregion.h"

#include <array>
#include <functional>
#include <optional>
#include <string>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/worldcityregion.cpp/.h.
 * EXE подтверждает 0x20-byte defence block, но `.city` задаёт только первые
 * пять DWORD, а исходный constructor не инициализирует последние три. Старый
 * сервер тем самым отправлял неопределённую память. Новая Linux-реализация
 * фиксирует неподтверждённые флаги в false/0: это безопасная техническая
 * нормализация недетерминированного дефекта, а не новая игровая семантика.
 */

class CWorldCityRegion final : public CWorldWarRegion
{
public:
    struct Build { std::array<std::uint8_t, 0x2C> header{}; std::string name; std::string script; };

    CWorldCityRegion() { SetSymbols(3, 3, 2); }
    bool AddToByteArray(std::vector<std::uint8_t>& output,
                        bool includeChild) const override;
    [[nodiscard]] bool LoadCitySetup(
        std::string_view text,
        const std::function<std::string(std::string_view)>& resolveName);
    void SetDefenceSetup(const Setup& setup) noexcept;
    [[nodiscard]] std::vector<Build>& Gates() noexcept { return m_Gates; }

private:
    std::array<std::optional<std::int32_t>, 8> m_DefenceSetup;
    std::vector<Build> m_Gates;
};
