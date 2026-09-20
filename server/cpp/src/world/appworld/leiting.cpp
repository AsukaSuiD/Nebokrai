#include "leiting.h"

std::optional<std::uint32_t> CLeiTing::Run(const LeiTingLocalTime& current) noexcept
{
    if (current.yearDay == m_Last.yearDay) return std::nullopt;
    const std::uint32_t kind = current.month == m_Last.month ? 1U : 2U;
    m_Last = current;
    return kind;
}
