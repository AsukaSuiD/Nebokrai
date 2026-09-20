#include "incrementlog.h"

#include <algorithm>

namespace { template<class T> void Append(std::vector<std::uint8_t>& out,const T value){const auto* p=reinterpret_cast<const std::uint8_t*>(&value);out.insert(out.end(),p,p+sizeof(value));} }

bool CIncrementLog::Add(const std::int32_t playerId, IncrementLogEntry entry)
{
    std::scoped_lock lock(m_Mutex); m_Entries[playerId].push_back(std::move(entry)); return true;
}
std::size_t CIncrementLog::Size(const std::int32_t playerId) const
{
    std::scoped_lock lock(m_Mutex); const auto found=m_Entries.find(playerId); return found==m_Entries.end()?0:found->second.size();
}
IncrementLogPage CIncrementLog::Page(const std::int32_t playerId, std::int32_t page) const
{
    std::scoped_lock lock(m_Mutex); if(page<0) page=0; IncrementLogPage result;
    const auto found=m_Entries.find(playerId); if(found==m_Entries.end()) return result;
    const auto& values=found->second; result.pageCount=static_cast<std::int32_t>((values.size()+16)/17);
    auto begin=static_cast<std::size_t>(page)*17; if(begin>=values.size()) begin=0;
    const auto end=std::min(begin+17,values.size()); result.entries.reserve(end-begin);
    for(auto index=end; index>begin; --index) result.entries.push_back(values[index-1]);
    return result;
}
bool CIncrementLog::SerializePage(std::vector<std::uint8_t>& output,const std::int32_t playerId,const std::int32_t page) const
{
    const auto value=Page(playerId,page); Append(output,static_cast<std::int32_t>(value.entries.size())); Append(output,value.pageCount);
    for(const auto& entry:value.entries){Append(output,entry.time);Append(output,entry.type);Append(output,entry.money);output.insert(output.end(),entry.description.begin(),entry.description.end());output.push_back(0);} return true;
}
