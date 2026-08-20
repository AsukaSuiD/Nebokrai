#include "goodswarmember.h"

#include <algorithm>

bool CGoodsWarMember::AddMember(const std::int32_t player,const std::int32_t faction){return m_Members.emplace(player,faction).second;}
bool CGoodsWarMember::RemoveMember(const std::int32_t player){return m_Members.erase(player)>0;}
std::size_t CGoodsWarMember::RemoveFaction(const std::int32_t faction){m_Factions.erase(faction);m_Counts.erase(faction);return std::erase_if(m_Members,[=](const auto& value){return value.second==faction;});}
bool CGoodsWarMember::AddFaction(const std::int32_t faction){m_Counts.try_emplace(faction,0);return m_Factions.insert(faction).second;}
bool CGoodsWarMember::FactionWin(const std::int32_t faction){if(!m_Factions.contains(faction))return false;++m_Counts[faction];return true;}
std::vector<GoodsWarFactionCount> CGoodsWarMember::Counts() const{std::vector<GoodsWarFactionCount> result;result.reserve(m_Counts.size());for(const auto [id,wins]:m_Counts)result.push_back({id,wins});return result;}
