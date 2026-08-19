#include "loginqueue.h"

#include <algorithm>
#include <utility>

namespace Login
{
TagPwdChecked::TagPwdChecked(std::int32_t socketId,
                             std::uint32_t clientIp,
                             std::vector<std::uint8_t> account,
                             std::vector<std::uint8_t> worldServer,
                             bool hasMatrix)
    : m_ClientIP(clientIp),
      m_SocketID(socketId),
      m_Account(std::move(account)),
      m_WorldServer(std::move(worldServer)),
      m_HasMatrix(hasMatrix)
{
}

std::int32_t TagPwdChecked::SocketID() const noexcept
{
    return m_SocketID;
}

std::uint32_t TagPwdChecked::ClientIP() const noexcept
{
    return m_ClientIP;
}

std::span<const std::uint8_t> TagPwdChecked::Account() const noexcept
{
    return m_Account;
}

std::span<const std::uint8_t> TagPwdChecked::WorldServer() const noexcept
{
    return m_WorldServer;
}

bool TagPwdChecked::HasMatrix() const noexcept
{
    return m_HasMatrix;
}

void CLoginQueue::PushBackPwdChecked(TagPwdChecked checked,
                                     const KickOutCallback& kickOut)
{
    std::lock_guard<std::mutex> lock(m_PwdCheckedMutex);
    const auto found = std::find_if(
        m_PwdChecked.begin(),
        m_PwdChecked.end(),
        [&](const TagPwdChecked& pending) {
            return std::equal(pending.Account().begin(),
                              pending.Account().end(),
                              checked.Account().begin(),
                              checked.Account().end());
        });
    if (found != m_PwdChecked.end()) {
        if (kickOut) {
            kickOut(found->Account());
        }
        m_PwdChecked.erase(found);
    }
    m_PwdChecked.push_back(std::move(checked));
}

std::size_t CLoginQueue::PendingPwdChecked() const
{
    std::lock_guard<std::mutex> lock(m_PwdCheckedMutex);
    return m_PwdChecked.size();
}
}
