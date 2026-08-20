#pragma once

#include "rssetup.h"

#include <cstdint>
#include <string>
#include <vector>

/*
 * Исходный владелец: dbaccess/worlddb/dbgoods.cpp/.h.
 * Таблицы player_goods и extend_properties, удаление старого набора перед
 * сохранением и порядок строк подтверждены Nworldserver.exe/PDB и Rust.
 * Внутренние CGoods-контейнеры не передаются в DB-слой: process-owner заранее
 * делает устойчивый снимок, а этот owner сохраняет только DB-представление.
 */
struct GoodsDbRow {
    std::int64_t instanceId{};
    std::uint32_t goodsId{};
    std::uint32_t goodsIndex{};
    std::int32_t playerId{};
    std::string name;
    std::uint32_t price{};
    std::uint32_t amount{};
    std::int32_t place{};
    std::int32_t position{};
};

struct GoodsAddonDbRow {
    std::int64_t goodsInstanceId{};
    std::int32_t type{};
    std::int32_t modifierValue1{};
    std::int32_t modifierValue2{};
};

class CDBGoods {
public:
    [[nodiscard]] static WorldDbResult Load(IWorldDbExecutor&, std::int32_t playerId);
    [[nodiscard]] static bool Replace(IWorldDbExecutor&,
                                      std::int32_t playerId,
                                      const std::vector<GoodsDbRow>&,
                                      const std::vector<GoodsAddonDbRow>&);
};
