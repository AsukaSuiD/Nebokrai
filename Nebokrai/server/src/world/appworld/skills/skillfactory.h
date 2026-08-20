#pragma once

#include "skill.h"

#include <cstdint>
#include <map>
#include <memory>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/skills/skillfactory.cpp/.h.
 * PDB/EXE подтверждают ordered skill-map, cache clear и length-prefixed
 * сериализацию каждого skill. Глобальный cache заменён owned factory.
 */
class CSkillFactory {
public:
    bool Add(std::unique_ptr<CSkill> skill);
    [[nodiscard]] const CSkill* Find(std::uint32_t id) const noexcept;
    void Clear() noexcept { m_Skills.clear(); }
    [[nodiscard]] bool Serialize(std::vector<std::uint8_t>& output) const;
    bool AddUsageName(std::string name, std::uint32_t value);
    [[nodiscard]] std::uint32_t StringToUsage(std::string_view value) const noexcept;
private:
    std::map<std::uint32_t, std::unique_ptr<CSkill>> m_Skills;
    std::map<std::string, std::uint32_t, std::less<>> m_UsageNames;
};
