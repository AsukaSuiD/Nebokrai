#include "cteamate.h"

#include "csession.h"

#include <algorithm>
#include <chrono>
#include <cstring>

namespace {
std::uint32_t Now() noexcept { return static_cast<std::uint32_t>(std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::steady_clock::now().time_since_epoch()).count()); }
template <class T> void Append(std::vector<std::uint8_t>& out, T value) { const auto* b = reinterpret_cast<const std::uint8_t*>(&value); out.insert(out.end(), b, b + sizeof(T)); }
template <class T> bool Read(std::span<const std::uint8_t> in, std::size_t& at, T& value) { if (at > in.size() || in.size() - at < sizeof(T)) return false; std::memcpy(&value, in.data() + at, sizeof(T)); at += sizeof(T); return true; }
bool ReadString(std::span<const std::uint8_t> in, std::size_t& at, std::string& value) { if (at > in.size()) return false; auto first = in.begin() + static_cast<std::ptrdiff_t>(at); auto nul = std::find(first, in.end(), std::uint8_t{0}); if (nul == in.end() || std::distance(first, nul) > 255) return false; value.assign(reinterpret_cast<const char*>(&*first), static_cast<std::size_t>(std::distance(first, nul))); at += value.size() + 1; return true; }
}

CTeamate::CTeamate(CSessionFactory& factory) : CPlug(factory) { SetPlugType(5); }

void CTeamate::SetOwnerRegionID(std::int32_t value)
{
    m_OwnerRegionId = value;
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    ChangeState(static_cast<std::int32_t>(State::ChangeRegion), {bytes, sizeof(value)});
}

bool CTeamate::IsPlugAvailable() const
{
    const auto now = Now();
    if (m_LastProbeAt == 0 || static_cast<std::uint32_t>(now - m_LastProbeAt) >= 60'000) {
        m_PlayerStillExists = !m_ExistenceProbe || m_ExistenceProbe(GetOwnerType(), GetOwnerID());
        m_LastProbeAt = now;
    }
    return m_PlayerStillExists;
}

bool CTeamate::OnChangeState(std::int32_t plugId, std::int32_t, std::span<const std::uint8_t>)
{
    CSession* session = GetSession();
    return session == nullptr || session->QueryPlugByID(plugId) != nullptr;
}

bool CTeamate::Serialize(std::vector<std::uint8_t>& output) const
{
    if (!CPlug::Serialize(output) || m_OwnerName.find('\0') != std::string::npos) return false;
    Append(output, m_OwnerRegionId); output.insert(output.end(), m_OwnerName.begin(), m_OwnerName.end()); output.push_back(0); return true;
}

bool CTeamate::Unserialize(std::span<const std::uint8_t> input, std::size_t& offset)
{
    return CPlug::Unserialize(input, offset) && Read(input, offset, m_OwnerRegionId) && ReadString(input, offset, m_OwnerName);
}
