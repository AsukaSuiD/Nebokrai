#include "skill.h"

#include <limits>

namespace {
template <class T> void Append(std::vector<std::uint8_t>& out, const T value)
{
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    out.insert(out.end(), bytes, bytes + sizeof(value));
}
void AppendCString(std::vector<std::uint8_t>& out, const std::string_view value)
{
    out.insert(out.end(), value.begin(), value.end()); out.push_back(0);
}
}

bool CSkill::SetName(const std::string_view value) { if (value.size() >= 256) return false; m_Name = value; return true; }
bool CSkill::SetDescription(const std::string_view value) { if (value.size() >= 256) return false; m_Description = value; return true; }

bool CSkill::Serialize(std::vector<std::uint8_t>& output) const
{
    if (m_Usages.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max())) return false;
    Append(output, m_Id); Append(output, m_Type); Append(output, m_Level);
    AppendCString(output, m_Name); AppendCString(output, m_Description);
    Append(output, static_cast<std::int32_t>(m_TargetSelf));
    Append(output, static_cast<std::int32_t>(m_Usages.size()));
    for (const auto usage : m_Usages) { Append(output, usage.type); Append(output, usage.cost); }
    return true;
}
