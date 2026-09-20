#include "cfairycontainer.h"

#include <stdexcept>

void CFairyContainer::SetHatchTime(std::size_t slot, std::uint32_t value)
{
    if (slot >= m_HatchTime.size()) {
        throw std::out_of_range("Неверная ячейка времени вылупления феи");
    }
    m_HatchTime[slot] = value;
}

std::uint32_t CFairyContainer::GetHatchTime(std::size_t slot) const
{
    if (slot >= m_HatchTime.size()) {
        throw std::out_of_range("Неверная ячейка времени вылупления феи");
    }
    return m_HatchTime[slot];
}
