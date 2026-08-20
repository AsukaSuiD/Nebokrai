#pragma once

#include "countryidentity.h"

#include <cstdint>

/*
 * Исходный владелец: WorldServer/appworld/country/officer.cpp/.h.
 * Четыре byte-флага следуют identity; разнородные старые union-имена сведены
 * к одному типизированному состоянию без изменения значений.
 */
class COfficer : public CCountryIdentity
{
public:
    [[nodiscard]] std::uint8_t GetJob() const noexcept { return m_Job; }
    void SetJob(std::uint8_t value) noexcept { m_Job = value; }
    [[nodiscard]] bool IsOnline() const noexcept { return m_Online; }
    void SetOnline(bool value) noexcept { m_Online = value; }
    [[nodiscard]] bool IsAppointed() const noexcept { return m_Appointed; }
    void SetAppointed(bool value) noexcept { m_Appointed = value; }
    [[nodiscard]] bool SalaryReceived() const noexcept { return m_SalaryReceived; }
    void SetSalaryReceived(bool value) noexcept { m_SalaryReceived = value; }

private:
    std::uint8_t m_Job{};
    bool m_Online{};
    bool m_Appointed{};
    bool m_SalaryReceived{};
};
