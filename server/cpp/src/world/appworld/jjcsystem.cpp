#include "jjcsystem.h"

#include <algorithm>

void CJJcSystem::ConfigureWeeklyReset(const std::int32_t day,const std::int32_t hour,const std::int32_t minute,const std::int32_t second) noexcept
{m_ResetDay=day;m_ResetHour=hour;m_ResetMinute=minute;m_ResetSecond=second;}

void CJJcSystem::ConfigureRegion(const std::int32_t region)
{if(region>0)m_AvailableRegions.insert(region);}

void CJJcSystem::ConfigureRegions(const std::int32_t first,const std::int32_t last)
{for(auto id=first;id<=last;++id)ConfigureRegion(id);}

void CJJcSystem::ConfigureLevelStep(const std::int32_t minimum,const std::int32_t maximum,const std::int32_t step)
{if(minimum<=maximum&&step>0)m_LevelSteps[{minimum,maximum}]=step;}

bool CJJcSystem::Apply(const std::int32_t player,JjcInfo info)
{if(m_Queue.contains(player)||std::ranges::any_of(m_Fights,[=](const auto& pair){return pair.second.firstPlayerId==player||pair.second.secondPlayerId==player;}))return false;return m_Queue.emplace(player,info).second;}

bool CJJcSystem::Quit(const std::int32_t player){return m_Queue.erase(player)>0;}

const JjcInfo* CJJcSystem::QueryPlayer(const std::int32_t player) const noexcept
{const auto value=m_Queue.find(player);return value==m_Queue.end()?nullptr:&value->second;}

const JjcFight* CJJcSystem::QueryFight(const std::int32_t region) const noexcept
{const auto value=m_Fights.find(region);return value==m_Fights.end()?nullptr:&value->second;}

std::int32_t CJJcSystem::LevelStep(const std::uint32_t level) const noexcept
{for(const auto& [range,step]:m_LevelSteps)if(level>=static_cast<std::uint32_t>(range.first)&&level<=static_cast<std::uint32_t>(range.second))return step;return 0;}

std::optional<std::int32_t> CJJcSystem::MatchOpponent(const std::int32_t player) const noexcept
{
    const auto own=m_Queue.find(player);if(own==m_Queue.end())return std::nullopt;const auto step=LevelStep(own->second.jjcLevel);
    for(const auto& [id,info]:m_Queue){if(id==player)continue;const auto distance=own->second.jjcLevel>info.jjcLevel?own->second.jjcLevel-info.jjcLevel:info.jjcLevel-own->second.jjcLevel;if(distance<=static_cast<std::uint32_t>(step))return id;}return std::nullopt;
}

std::optional<std::int32_t> CJJcSystem::AcquireRegion()
{if(m_AvailableRegions.empty())return std::nullopt;const auto id=*m_AvailableRegions.begin();m_AvailableRegions.erase(m_AvailableRegions.begin());m_UsedRegions.insert(id);return id;}

bool CJJcSystem::ReleaseRegion(const std::int32_t region) noexcept
{
    if(m_Fights.contains(region)||m_UsedRegions.erase(region)==0)return false;
    m_AvailableRegions.insert(region);return true;
}

bool CJJcSystem::StartFight(const std::int32_t region,const std::int32_t first,const std::int32_t second,const std::int32_t now)
{
    if(!m_UsedRegions.contains(region)||!m_Queue.contains(first)||!m_Queue.contains(second)||m_Fights.contains(region))return false;
    auto& firstInfo=m_Queue.at(first);auto& secondInfo=m_Queue.at(second);
    firstInfo.opponentId=second;secondInfo.opponentId=first;
    firstInfo.jjcRegionId=region;secondInfo.jjcRegionId=region;
    firstInfo.startTime=now;secondInfo.startTime=now;
    return m_Fights.emplace(region,JjcFight{region,first,second,now}).second;
}

bool CJJcSystem::EndFight(const std::int32_t region)
{
    const auto fight=m_Fights.find(region);if(fight==m_Fights.end())return false;
    m_Queue.erase(fight->second.firstPlayerId);m_Queue.erase(fight->second.secondPlayerId);
    m_Fights.erase(fight);m_UsedRegions.erase(region);m_AvailableRegions.insert(region);return true;
}

CJJcSystem::RunResult CJJcSystem::Run(const std::int32_t now,const std::int32_t day,const std::int32_t hour,const std::int32_t minute,const std::int32_t timeout)
{
    RunResult result;const bool gate=day==m_ResetDay&&hour==m_ResetHour&&minute==m_ResetMinute;
    if(gate&&!m_UpdatedThisWeek){result.weeklyReset=true;++m_PassedWeeks;result.seasonReset=m_PassedWeeks%4==0;m_UpdatedThisWeek=true;}
    if(day!=0)m_UpdatedThisWeek=false; // сохранён исходный жёсткий Sunday reset.
    for(auto position=m_Fights.begin();position!=m_Fights.end();){if(now-position->second.startedAt>timeout){result.timedOut.push_back(position->second);result.recycledRegions.push_back(position->first);m_UsedRegions.erase(position->first);m_AvailableRegions.insert(position->first);position=m_Fights.erase(position);}else ++position;}
    return result;
}
