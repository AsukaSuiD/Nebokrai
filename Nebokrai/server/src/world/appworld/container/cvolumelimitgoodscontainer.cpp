#include "cvolumelimitgoodscontainer.h"

#include <algorithm>

void CVolumeLimitGoodsContainer::SetContainerVolume(std::uint32_t size)
{
    Release();
    m_Size = size;
    m_Cells.assign(size, CGUID::GUID_INVALID);
    SetGoodsAmountLimit(size);
}

void CVolumeLimitGoodsContainer::SetContainerVolume(std::uint32_t rows,
                                                     std::uint32_t columns)
{
    SetContainerVolume(rows * columns);
}

void CVolumeLimitGoodsContainer::Clear(void* context)
{
    CAmountLimitGoodsContainer::Clear(context);
    m_Cells.assign(m_Size, CGUID::GUID_INVALID);
}

void CVolumeLimitGoodsContainer::Release(void* context)
{
    CAmountLimitGoodsContainer::Release(context);
    m_Size = 0;
    m_Cells.clear();
}

bool CVolumeLimitGoodsContainer::Add(GoodsPtr& goods, void* context)
{
    std::uint32_t position{};
    return FindFreePosition(position) && AddAt(position, goods, context);
}

bool CVolumeLimitGoodsContainer::AddAt(std::uint32_t position,
                                       GoodsPtr& goods,
                                       void* context)
{
    if (!goods || !IsSpaceEnough(position)) {
        return false;
    }
    const CGUID guid = goods->GetGUID();
    if (!CAmountLimitGoodsContainer::Add(goods, context)) {
        return false;
    }
    m_Cells[position] = guid;
    return true;
}

bool CVolumeLimitGoodsContainer::AddFromDB(GoodsPtr& goods,
                                           std::uint32_t position,
                                           void* context)
{
    return AddAt(position, goods, context);
}

CGoodsContainer::GoodsPtr CVolumeLimitGoodsContainer::Remove(const CGUID& guid,
                                                              void* context)
{
    std::uint32_t position{};
    if (QueryGoodsPosition(guid, position)) {
        m_Cells[position] = CGUID::GUID_INVALID;
    }
    return CAmountLimitGoodsContainer::Remove(guid, context);
}

CGoods* CVolumeLimitGoodsContainer::GetGoods(std::uint32_t position) const noexcept
{
    return position < m_Cells.size() && !m_Cells[position].IsInvalided()
               ? Find(m_Cells[position])
               : nullptr;
}

bool CVolumeLimitGoodsContainer::QueryGoodsPosition(const CGUID& guid,
                                                     std::uint32_t& position) const noexcept
{
    const auto found = std::ranges::find(m_Cells, guid);
    if (found == m_Cells.end()) {
        return false;
    }
    position = static_cast<std::uint32_t>(found - m_Cells.begin());
    return true;
}

bool CVolumeLimitGoodsContainer::IsSpaceEnough(std::uint32_t position) const noexcept
{
    return position < m_Cells.size() && m_Cells[position].IsInvalided();
}

bool CVolumeLimitGoodsContainer::FindFreePosition(std::uint32_t& position) const noexcept
{
    const auto found = std::ranges::find(m_Cells, CGUID::GUID_INVALID);
    if (found == m_Cells.end()) {
        return false;
    }
    position = static_cast<std::uint32_t>(found - m_Cells.begin());
    return true;
}
