#include "skillfactory.h"

#include <limits>

namespace { template<class T> void Append(std::vector<std::uint8_t>& out, const T v) { const auto* p=reinterpret_cast<const std::uint8_t*>(&v); out.insert(out.end(),p,p+sizeof(v)); } }

bool CSkillFactory::Add(std::unique_ptr<CSkill> skill)
{
    if (!skill) return false;
    return m_Skills.emplace(skill->GetID(), std::move(skill)).second;
}
const CSkill* CSkillFactory::Find(const std::uint32_t id) const noexcept
{
    const auto found=m_Skills.find(id); return found==m_Skills.end()?nullptr:found->second.get();
}
bool CSkillFactory::Serialize(std::vector<std::uint8_t>& output) const
{
    if(m_Skills.size()>static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max())) return false;
    Append(output,static_cast<std::int32_t>(m_Skills.size()));
    for(const auto& [id,skill]:m_Skills){(void)id; std::vector<std::uint8_t> bytes; if(!skill->Serialize(bytes)) return false; Append(output,static_cast<std::int32_t>(bytes.size())); output.insert(output.end(),bytes.begin(),bytes.end());}
    return true;
}
bool CSkillFactory::AddUsageName(std::string name, const std::uint32_t value)
{
    return m_UsageNames.emplace(std::move(name), value).second;
}
std::uint32_t CSkillFactory::StringToUsage(const std::string_view value) const noexcept
{
    const auto found = m_UsageNames.find(value);
    return found == m_UsageNames.end() ? 0 : found->second;
}
