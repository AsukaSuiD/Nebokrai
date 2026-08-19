#include "mysocket.h"

std::int32_t SocketIdAllocator::Next() noexcept
{
    const std::uint32_t next = m_LastIssued.fetch_add(1U, std::memory_order_relaxed) + 1U;
    return std::bit_cast<std::int32_t>(next);
}
