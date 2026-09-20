#include "monster.h"

#include <utility>

CMonster::CMonster()
{
    SetType(600);
}

CMonster::CMonster(std::string originalName) : m_OriginalName(std::move(originalName))
{
    SetType(600);
}

std::uint8_t CMonster::GetFigure() const noexcept
{
    return m_FigureResolver ? m_FigureResolver(m_OriginalName) : 0;
}
