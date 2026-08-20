#include "csessionfactory.h"

#include "cplug.h"
#include "csession.h"
#include "cteam.h"
#include "cteamate.h"

#include <cstring>

namespace {
template <class T> bool Read(std::span<const std::uint8_t> input, std::size_t& offset, T& value) {
    if (offset > input.size() || input.size() - offset < sizeof(T)) return false;
    std::memcpy(&value, input.data() + offset, sizeof(T)); offset += sizeof(T); return true;
}
}

CSession* CSessionFactory::QuerySession(std::int32_t id) const noexcept
{
    const auto found = m_Sessions.find(id); return found == m_Sessions.end() ? nullptr : found->second.get();
}

CPlug* CSessionFactory::QueryPlug(std::int32_t id) const noexcept
{
    const auto found = m_Plugs.find(id); return found == m_Plugs.end() ? nullptr : found->second.get();
}

std::int32_t CSessionFactory::CreateSession(std::uint32_t minimumPlugs,
                                            std::uint32_t maximumPlugs,
                                            std::uint32_t lifetimeMs,
                                            SessionType type)
{
    std::unique_ptr<CSession> session;
    if (type == SessionType::Normal) session = std::make_unique<CSession>(*this, minimumPlugs, maximumPlugs, lifetimeMs);
    else if (type == SessionType::Team) session = std::make_unique<CTeam>(*this, minimumPlugs, maximumPlugs, lifetimeMs);
    else return 0;
    const auto id = m_NextSessionId++;
    session->SetType(static_cast<std::int32_t>(ObjectType::Session)); session->SetID(id);
    m_Sessions[id] = std::move(session); return id;
}

std::int32_t CSessionFactory::CreatePlug(PlugType type, std::int32_t ownerType, std::int32_t ownerId)
{
    if (type != PlugType::Teamate) return 0;
    auto plug = std::make_unique<CTeamate>(*this);
    const auto id = m_NextPlugId++;
    plug->SetType(static_cast<std::int32_t>(ObjectType::Plug)); plug->SetID(id); plug->SetOwner(ownerType, ownerId);
    m_Plugs[id] = std::move(plug); return id;
}

bool CSessionFactory::InsertPlug(std::int32_t sessionId, std::int32_t plugId)
{
    CSession* session = QuerySession(sessionId); return session != nullptr && session->InsertPlug(plugId);
}

bool CSessionFactory::GarbageCollect(ObjectType type, std::int32_t id)
{
    if (type == ObjectType::Session) return m_Sessions.erase(id) != 0;
    if (type == ObjectType::Plug) return m_Plugs.erase(id) != 0;
    return false;
}

void CSessionFactory::AI()
{
    for (auto it = m_Sessions.begin(); it != m_Sessions.end();) {
        CSession& session = *it->second;
        if (!session.IsSessionAvailable()) { session.Abort(); it = m_Sessions.erase(it); }
        else if (session.IsSessionEnded()) { session.End(); it = m_Sessions.erase(it); }
        else { session.AI(); ++it; }
    }
}

std::int32_t CSessionFactory::UnserializeSession(std::span<const std::uint8_t> input, std::size_t& offset)
{
    std::int32_t rawType{}; std::uint32_t minimum{}, maximum{}, lifetime{};
    if (!Read(input, offset, rawType) || !Read(input, offset, minimum) || !Read(input, offset, maximum) || !Read(input, offset, lifetime) || rawType < 0 || rawType > 1) return 0;
    const auto id = CreateSession(minimum, maximum, lifetime, static_cast<SessionType>(rawType));
    CSession* session = QuerySession(id);
    if (session == nullptr || !session->Unserialize(input, offset)) { GarbageCollect(ObjectType::Session, id); return 0; }
    return id;
}

std::int32_t CSessionFactory::UnserializePlug(std::span<const std::uint8_t> input, std::size_t& offset)
{
    std::int32_t rawType{}, ownerType{}, ownerId{};
    if (!Read(input, offset, rawType) || !Read(input, offset, ownerType) || !Read(input, offset, ownerId) || rawType < 0 || rawType > 5) return 0;
    const auto id = CreatePlug(static_cast<PlugType>(rawType), ownerType, ownerId);
    CPlug* plug = QueryPlug(id);
    if (plug == nullptr || !plug->Unserialize(input, offset)) { GarbageCollect(ObjectType::Plug, id); return 0; }
    return id;
}
