#pragma once

#include <array>
#include <cstdint>

namespace Auction
{
/*
 * Исходный владелец: public/auctionroom/auctionroom.h:56.
 * Точный PDB и сообщение 0x14ED09 подтверждают порядок 71 DWORD и значения
 * конструктора. Имя остаётся фиксированным 256-байтовым legacy-буфером.
 */
struct PlayerOptNode
{
    std::uint32_t playerId{};
    std::uint32_t currentPage{};
    std::int32_t lowLevel{};
    std::int32_t upLevel{999};
    std::int32_t useSelf{1};
    std::int32_t moneyType{1};
    std::int32_t weaponType{-1};
    std::array<std::uint8_t, 256> goodsName{};
};
}
