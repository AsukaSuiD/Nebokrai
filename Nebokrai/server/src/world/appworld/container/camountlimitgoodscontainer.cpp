#include "camountlimitgoodscontainer.h"

#include <algorithm>

bool CAmountLimitGoodsContainer::Add(GoodsPtr& goods, void* context)
{
    return !IsFull() && CGoodsContainer::Add(goods, context);
}

void CAmountLimitGoodsContainer::Clear(void* context)
{
    CGoodsContainer::Clear(context);
    m_LockedGoods.clear();
}

void CAmountLimitGoodsContainer::Release(void* context)
{
    CGoodsContainer::Release(context);
    m_GoodsAmountLimit = 1;
    m_LockedGoods.clear();
}

bool CAmountLimitGoodsContainer::Lock(const CGUID& guid)
{
    if (Find(guid) == nullptr || IsLocked(guid)) {
        return false;
    }
    m_LockedGoods.push_back(guid);
    return true;
}

bool CAmountLimitGoodsContainer::Unlock(const CGUID& guid) noexcept
{
    return std::erase(m_LockedGoods, guid) != 0;
}

bool CAmountLimitGoodsContainer::IsLocked(const CGUID& guid) const noexcept
{
    return std::ranges::find(m_LockedGoods, guid) != m_LockedGoods.end();
}

std::uint32_t CAmountLimitGoodsContainer::GetContentsWeight() const noexcept
{
    std::uint32_t weight{};
    for (const auto& [guid, goods] : Goods()) {
        (void)guid;
        weight += goods->GetWeight(Factory().Query(goods->GetBasePropertiesIndex()));
    }
    return weight;
}
