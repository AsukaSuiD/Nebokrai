#include "csession.h"

#include "cplug.h"
#include "csessionfactory.h"

#include <algorithm>
#include <chrono>

namespace {
template <class T> void Append(std::vector<std::uint8_t>& out, T value) {
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    out.insert(out.end(), bytes, bytes + sizeof(T));
}
}

CSession::CSession(CSessionFactory& factory,
                   std::uint32_t minimumPlugs,
                   std::uint32_t maximumPlugs,
                   std::uint32_t lifetimeMs) noexcept
    : m_Factory(factory), m_MaximumPlugs(maximumPlugs),
      m_MinimumPlugs(minimumPlugs), m_LifetimeMs(lifetimeMs)
{
}

std::uint32_t CSession::Now() noexcept
{
    const auto value = std::chrono::duration_cast<std::chrono::milliseconds>(
        std::chrono::steady_clock::now().time_since_epoch()).count();
    return static_cast<std::uint32_t>(value);
}

bool CSession::IsSessionAvailable() const
{
    return m_Plugs.size() >= m_MinimumPlugs;
}

CPlug* CSession::QueryPlugByID(std::int32_t plugId) const noexcept
{
    return std::ranges::find(m_Plugs, plugId) == m_Plugs.end() ? nullptr : m_Factory.QueryPlug(plugId);
}

CPlug* CSession::QueryPlugByOwner(std::int32_t ownerType, std::int32_t ownerId) const noexcept
{
    for (const auto id : m_Plugs) {
        CPlug* plug = m_Factory.QueryPlug(id);
        if (plug != nullptr && plug->GetOwnerType() == ownerType && plug->GetOwnerID() == ownerId) return plug;
    }
    return nullptr;
}

bool CSession::InsertPlug(std::int32_t plugId)
{
    if (m_Started || m_Plugs.size() >= m_MaximumPlugs || QueryPlugByID(plugId) != nullptr) return false;
    CPlug* plug = m_Factory.QueryPlug(plugId);
    if (plug == nullptr || !plug->IsPlugAvailable()) return false;
    plug->SetSession(GetID());
    m_Plugs.push_back(plugId);
    return OnPlugInserted(plugId);
}

bool CSession::Start()
{
    m_StartedAt = Now(); m_Started = true;
    return OnSessionStarted();
}

bool CSession::End()
{
    m_StartedAt = 0; m_Ended = true;
    for (const auto id : m_Plugs) if (auto* plug = m_Factory.QueryPlug(id)) plug->OnSessionEnded();
    return OnSessionEnded();
}

bool CSession::Abort()
{
    m_StartedAt = 0; m_Aborted = true;
    for (const auto id : m_Plugs) if (auto* plug = m_Factory.QueryPlug(id)) plug->OnSessionAborted();
    return OnSessionAborted();
}

bool CSession::OnPlugInserted(std::int32_t plugId)
{
    CPlug* plug = m_Factory.QueryPlug(plugId);
    if (plug != nullptr) plug->OnPlugInserted();
    OnPlugChangeState(plugId, 0, {}, false);
    return plug != nullptr;
}

bool CSession::OnPlugEnded(std::int32_t plugId)
{
    CPlug* plug = m_Factory.QueryPlug(plugId);
    if (plug == nullptr) return false;
    plug->OnPlugEnded(); OnPlugChangeState(plugId, 1, {}, false);
    m_Factory.GarbageCollect(CSessionFactory::ObjectType::Plug, plugId);
    return true;
}

bool CSession::OnPlugAborted(std::int32_t plugId)
{
    CPlug* plug = m_Factory.QueryPlug(plugId);
    if (plug == nullptr) return false;
    plug->OnPlugAborted(); OnPlugChangeState(plugId, 2, {}, false);
    m_Factory.GarbageCollect(CSessionFactory::ObjectType::Plug, plugId);
    return true;
}

bool CSession::OnPlugChangeState(std::int32_t plugId,
                                 std::int32_t state,
                                 std::span<const std::uint8_t> payload,
                                 bool includeSender)
{
    CPlug* source = QueryPlugByID(plugId);
    if (source == nullptr || !source->IsPlugAvailable()) return false;
    for (const auto id : m_Plugs) {
        if (!includeSender && id == plugId) continue;
        CPlug* target = m_Factory.QueryPlug(id);
        if (target != nullptr && target->IsPlugAvailable() && !target->IsPlugEnded())
            target->OnChangeState(plugId, state, payload);
    }
    return true;
}

bool CSession::Serialize(std::vector<std::uint8_t>& output) const
{
    std::uint32_t remaining{};
    if (m_LifetimeMs != 0 && m_StartedAt != 0) remaining = m_StartedAt + m_LifetimeMs - Now();
    Append(output, m_SessionType); Append(output, m_MinimumPlugs);
    Append(output, m_MaximumPlugs); Append(output, remaining);
    return true;
}

void CSession::AI()
{
    if (m_Started && !m_Ended && m_LifetimeMs != 0 &&
        static_cast<std::uint32_t>(Now() - m_StartedAt) >= m_LifetimeMs) m_Ended = true;
}
