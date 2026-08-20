#include "cteam.h"

#include "cplug.h"
#include "csessionfactory.h"

#include <algorithm>
#include <cstring>

namespace {
template <class T> void Append(std::vector<std::uint8_t>& out, T value) { const auto* b = reinterpret_cast<const std::uint8_t*>(&value); out.insert(out.end(), b, b + sizeof(T)); }
template <class T> bool Read(std::span<const std::uint8_t> in, std::size_t& at, T& value) { if (at > in.size() || in.size() - at < sizeof(T)) return false; std::memcpy(&value, in.data() + at, sizeof(T)); at += sizeof(T); return true; }
bool AppendString(std::vector<std::uint8_t>& out, std::string_view value) { if (value.find('\0') != std::string_view::npos) return false; out.insert(out.end(), value.begin(), value.end()); out.push_back(0); return true; }
bool ReadString(std::span<const std::uint8_t> in, std::size_t& at, std::string& value) { if (at > in.size()) return false; auto first = in.begin() + static_cast<std::ptrdiff_t>(at); auto nul = std::find(first, in.end(), std::uint8_t{0}); if (nul == in.end() || std::distance(first, nul) > 255) return false; value.assign(reinterpret_cast<const char*>(&*first), static_cast<std::size_t>(std::distance(first, nul))); at += value.size() + 1; return true; }
}

CTeam::CTeam(CSessionFactory& factory, std::uint32_t minimumPlugs, std::uint32_t maximumPlugs, std::uint32_t lifetimeMs)
    : CSession(factory, minimumPlugs, maximumPlugs, lifetimeMs)
{
    SetSessionType(1);
}

bool CTeam::IsSessionAvailable() const
{
    return m_DelayTicks > 0 || CSession::IsSessionAvailable();
}

void CTeam::AI()
{
    CSession::AI();
    if (m_DelayTicks > 0) --m_DelayTicks;
    if (m_LeaderId == 0 && !Plugs().empty()) {
        if (const CPlug* plug = QueryPlugByID(Plugs().front())) m_LeaderId = plug->GetOwnerID();
    }
}

bool CTeam::SetLeader(std::int32_t playerId)
{
    CPlug* plug = QueryPlugByOwner(400, playerId);
    if (Started() || plug == nullptr) return false;
    m_LeaderId = playerId;
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&playerId);
    return OnPlugChangeState(plug->GetID(), 4, {bytes, sizeof(playerId)}, true);
}

bool CTeam::KickPlayer(const std::int32_t playerId)
{
    CPlug* plug = QueryPlugByOwner(400, playerId);
    return plug != nullptr && plug->Exit();
}

void CTeam::SetAllocationScheme(Allocation value)
{
    m_Allocation = value;
    const auto raw = static_cast<std::int32_t>(value);
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&raw);
    if (CPlug* leader = QueryPlugByOwner(400, m_LeaderId)) OnPlugChangeState(leader->GetID(), 5, {bytes, sizeof(raw)}, true);
}

bool CTeam::Serialize(std::vector<std::uint8_t>& output) const
{
    if (!CSession::Serialize(output)) return false;
    Append(output, m_TeamId);
    if (!AppendString(output, m_TeamName) || !AppendString(output, m_Password)) return false;
    Append(output, m_LeaderId); Append(output, static_cast<std::uint32_t>(Plugs().size()));
    for (const auto id : Plugs()) {
        const CPlug* plug = QueryPlugByID(id); if (plug == nullptr || !plug->Serialize(output)) return false;
    }
    Append(output, static_cast<std::int32_t>(m_Allocation));
    return true;
}

bool CTeam::Unserialize(std::span<const std::uint8_t> input, std::size_t& offset)
{
    std::uint32_t plugCount{}; std::int32_t allocation{};
    if (!Read(input, offset, m_TeamId) || !ReadString(input, offset, m_TeamName) || !ReadString(input, offset, m_Password) || !Read(input, offset, m_LeaderId) || !Read(input, offset, plugCount)) return false;
    for (std::uint32_t i{}; i < plugCount; ++i) {
        const auto plugId = Factory().UnserializePlug(input, offset); if (plugId == 0 || !Factory().InsertPlug(GetID(), plugId)) return false;
    }
    if (!Read(input, offset, allocation) || allocation < 0 || allocation > 1) return false;
    m_Allocation = static_cast<Allocation>(allocation); return true;
}
