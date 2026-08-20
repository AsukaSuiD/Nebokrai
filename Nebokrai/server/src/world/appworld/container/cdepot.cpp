#include "cdepot.h"

/* Исходный владелец: cdepot.cpp/.h. Lock блокирует mutation, но не чтение. */
bool CDepot::Add(GoodsPtr& goods, void* context)
{
    return !m_Locked && CVolumeLimitGoodsContainer::Add(goods, context);
}

bool CDepot::AddAt(std::uint32_t position, GoodsPtr& goods, void* context)
{
    return !m_Locked && CVolumeLimitGoodsContainer::AddAt(position, goods, context);
}

CGoodsContainer::GoodsPtr CDepot::Remove(const CGUID& guid, void* context)
{
    return m_Locked ? GoodsPtr{} : CVolumeLimitGoodsContainer::Remove(guid, context);
}
