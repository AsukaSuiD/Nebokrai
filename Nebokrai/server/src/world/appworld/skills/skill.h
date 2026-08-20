#pragma once

#include <cstdint>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/skills/skill.cpp/.h.
 * Порядок полей и usage-пар в Serialize подтверждён декомпилятом. Фиксирован-
 * ные char[256] представлены bounded std::string без старого переполнения.
 */
class CSkill {
public:
    struct Usage { std::uint32_t type{}, cost{}; };
    explicit CSkill(std::uint32_t id = 0) noexcept : m_Id(id) {}
    bool SetName(std::string_view value);
    bool SetDescription(std::string_view value);
    void SetType(std::uint32_t value) noexcept { m_Type = value; }
    void SetLevel(std::uint32_t value) noexcept { m_Level = value; }
    void SetTargetSelf(bool value) noexcept { m_TargetSelf = value; }
    void AddUsage(Usage value) { m_Usages.push_back(value); }
    [[nodiscard]] std::uint32_t GetID() const noexcept { return m_Id; }
    [[nodiscard]] bool Serialize(std::vector<std::uint8_t>& output) const;

private:
    std::vector<Usage> m_Usages;
    std::string m_Name, m_Description;
    std::uint32_t m_Type{}, m_Id{}, m_Level{};
    bool m_TargetSelf{};
};
