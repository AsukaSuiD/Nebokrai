#include "cseekgoodslistener.h"

#include "../goods/cgoods.h"

void CSeekGoodsListener::OnTraversingContainer(CContainer&, CBaseObject& object)
{
    const auto* goods = dynamic_cast<CGoods*>(&object);
    if (goods != nullptr && m_Target && goods->GetBasePropertiesIndex() == *m_Target) m_Goods.push_back(goods->GetGUID());
}
