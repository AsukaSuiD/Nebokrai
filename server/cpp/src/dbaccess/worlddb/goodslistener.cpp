#include "goodslistener.h"

#include <iterator>

void CGoodsDbListener::Add(GoodsDbRow row, std::vector<GoodsAddonDbRow> addons)
{
    row.playerId = m_PlayerId;
    m_Goods.push_back(std::move(row));
    m_Addons.insert(m_Addons.end(),
                    std::make_move_iterator(addons.begin()),
                    std::make_move_iterator(addons.end()));
}

bool CGoodsDbListener::Save(IWorldDbExecutor& database) const
{
    return CDBGoods::Replace(database, m_PlayerId, m_Goods, m_Addons);
}
