#include "playerranks.h"

#include <limits>

namespace {template<class T>void Append(std::vector<std::uint8_t>& out,const T v){const auto*p=reinterpret_cast<const std::uint8_t*>(&v);out.insert(out.end(),p,p+sizeof(v));}void Text(std::vector<std::uint8_t>&out,const std::string&v){out.insert(out.end(),v.begin(),v.end());out.push_back(0);}}
bool CPlayerRanks::Add(PlayerRank rank){if(m_Maximum&&m_Ranks.size()>=m_Maximum)return false;m_Ranks.push_back(std::move(rank));return true;}
void CPlayerRanks::Replace(std::vector<PlayerRank> ranks){if(m_Maximum&&ranks.size()>m_Maximum)ranks.resize(m_Maximum);m_Ranks=std::move(ranks);}
bool CPlayerRanks::Serialize(std::vector<std::uint8_t>& output) const{if(m_Ranks.size()>static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()))return false;Append(output,static_cast<std::int32_t>(m_Ranks.size()));for(const auto&r:m_Ranks){Append(output,r.playerId);Text(output,r.name);Append(output,r.occupation);Append(output,r.level);Text(output,r.factionName);}return true;}
