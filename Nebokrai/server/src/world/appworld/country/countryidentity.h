#pragma once

#include <cstdint>
#include <string>
#include <string_view>

/*
 * Исходный владелец: WorldServer/appworld/country/countryidentity.cpp/.h.
 * Identity состоит из signed ID и byte-string имени; defaults подтверждены
 * PDB/raw constructor. std::string заменяет только MSVC string ABI.
 */
class CCountryIdentity
{
public:
    virtual ~CCountryIdentity() = default;
    [[nodiscard]] std::int32_t GetID() const noexcept { return m_Id; }
    [[nodiscard]] std::string_view GetName() const noexcept { return m_Name; }
    void SetID(std::int32_t value) noexcept { m_Id = value; }
    void SetName(std::string_view value);

private:
    std::int32_t m_Id{};
    std::string m_Name;
};
