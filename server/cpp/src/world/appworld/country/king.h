#pragma once

#include "officer.h"

#include <algorithm>
#include <cstdint>

/*
 * Исходный владелец: WorldServer/appworld/country/king.cpp/.h.
 * Три king point, назначение/salary flags и clamp к CountryParam подтверждены
 * PDB/raw. Максимумы передаются явно вместо process-global singleton.
 */
class CKing final : public COfficer
{
public:
    void SetControlPoint(std::int32_t value, std::int32_t maximum) noexcept
    { m_ControlPoint = std::min(value, maximum); }
    void SetMaterialPoint(std::int32_t value, std::int32_t maximum) noexcept
    { m_MaterialPoint = std::min(value, maximum); }
    void SetWarPoint(std::int32_t value, std::int32_t maximum) noexcept
    { m_WarPoint = std::min(value, maximum); }
    void ChangeControlPoint(std::int32_t delta, std::int32_t maximum) noexcept
    { m_ControlPoint = std::min(m_ControlPoint + delta, maximum); }
    [[nodiscard]] std::int32_t GetControlPoint() const noexcept { return m_ControlPoint; }
    [[nodiscard]] std::int32_t GetMaterialPoint() const noexcept { return m_MaterialPoint; }
    [[nodiscard]] std::int32_t GetWarPoint() const noexcept { return m_WarPoint; }
    [[nodiscard]] bool IsRegistered() const noexcept { return m_Registered; }
    void SetRegistered(bool value) noexcept { m_Registered = value; }

private:
    std::int32_t m_ControlPoint{};
    std::int32_t m_MaterialPoint{};
    std::int32_t m_WarPoint{};
    bool m_Registered{};
};
