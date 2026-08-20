#include "cplug.h"

#include "csession.h"
#include "csessionfactory.h"

#include <cstring>

namespace {
template <class T> void Append(std::vector<std::uint8_t>& out, T value) {
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    out.insert(out.end(), bytes, bytes + sizeof(T));
}
}

CPlug::CPlug(CSessionFactory& factory) noexcept : m_Factory(factory) {}

void CPlug::SetOwner(std::int32_t type, std::int32_t id) noexcept
{
    m_OwnerType = type;
    m_OwnerId = id;
}

CSession* CPlug::GetSession() const noexcept
{
    return m_Factory.QuerySession(m_SessionId);
}

bool CPlug::ChangeState(std::int32_t state, std::span<const std::uint8_t> payload)
{
    CSession* session = GetSession();
    return session != nullptr && session->OnPlugChangeState(GetID(), state, payload, false);
}

bool CPlug::Exit()
{
    CSession* session = GetSession();
    if (session == nullptr || !session->OnPlugChangeState(GetID(), 1, {}, false)) {
        return false;
    }
    m_Ended = true;
    return true;
}

bool CPlug::Serialize(std::vector<std::uint8_t>& output) const
{
    Append(output, m_PlugType);
    Append(output, m_OwnerType);
    Append(output, m_OwnerId);
    Append(output, static_cast<std::int32_t>(m_Ended));
    return true;
}

bool CPlug::Unserialize(std::span<const std::uint8_t> input, std::size_t& offset)
{
    std::int32_t ended{};
    if (offset > input.size() || input.size() - offset < sizeof(ended)) return false;
    std::memcpy(&ended, input.data() + offset, sizeof(ended));
    offset += sizeof(ended);
    m_Ended = ended != 0;
    return true;
}
