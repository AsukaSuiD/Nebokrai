#pragma once

#include "dbgoods.h"

#include <cstdint>
#include <vector>

/*
 * Исходный владелец: dbaccess/worlddb/goodslistener.cpp/.h.
 * В оригинале listener обходил все player-контейнеры и немедленно писал их
 * через CDBGoods. Здесь обход остаётся у контейнеров, а listener только
 * собирает плоский снимок и одним вызовом передаёт его DB-owner-у.
 */
class CGoodsDbListener {
public:
    explicit CGoodsDbListener(std::int32_t playerId) noexcept : m_PlayerId(playerId) {}
    void Add(GoodsDbRow row, std::vector<GoodsAddonDbRow> addons = {});
    [[nodiscard]] bool Save(IWorldDbExecutor&) const;
    [[nodiscard]] std::size_t Size() const noexcept { return m_Goods.size(); }

private:
    std::int32_t m_PlayerId{};
    std::vector<GoodsDbRow> m_Goods;
    std::vector<GoodsAddonDbRow> m_Addons;
};
