#include "miscservermessage.h"

#include "game.h"

#include "../public/guid.h"

#include <algorithm>
#include <bit>
#include <map>

#include <spdlog/spdlog.h>

namespace Misc
{
namespace
{
constexpr std::uint32_t kSyncWaitMilliseconds = 120'000;

CGUID ReadGuid(CBaseMessage& message)
{
    CGUID guid;
    if (!message.GetGUID(guid)) return CGUID::GUID_INVALID;
    return guid;
}
}

void OnMSG_W2M_AUCTION(MiscNet::CMessage& message, CGame& game)
{
    switch (message.MessageType()) {
    case 0x0014'ED01:
    {
        game.CountNewAuctionItem();
        auto item = std::make_unique<Auction::CGoodsNode>();
        std::size_t cursor = static_cast<std::size_t>(message.Base().GetReadPtr());
        const auto unserialize = item->UnSerialize(message.Base().MessageData(), cursor);
        message.Base().SetReadPtr(static_cast<std::int32_t>(cursor));
        if (unserialize) return;
        bool added{};
        static_cast<void>(game.AddAuctionItem(std::move(item), added));
        return;
    }
    case 0x0014'ED04:
    {
        const std::int32_t playerId = message.Base().GetLong();
        const CGUID guid = ReadGuid(message.Base());
        const bool pushed = game.AuctionRoom().PushItemToOptList(guid, 3, playerId);
        if (pushed) return;
        MiscNet::CMessage response(0x0015'EB06);
        response.Base().Add(std::int32_t{0});
        response.Base().Add(playerId);
        static_cast<void>(response.Send(game.SendQueue(), false));
        return;
    }
    case 0x0014'ED05:
    {
        const auto doneCount = game.AuctionSyncCount();
        if (!doneCount || *doneCount != 0) return;
        const std::uint8_t mapId = message.Base().GetByte();
        const std::uint32_t count =
            std::bit_cast<std::uint32_t>(message.Base().GetLong());
        if (count > 10'000U) {
            spdlog::warn(
                "MiscServer: WorldServer прислал большой список сверки аукциона: {}",
                count);
        }
        std::map<CGUID, bool> existing;
        for (std::uint32_t index = 0; index < count; ++index) {
            existing.emplace(ReadGuid(message.Base()), false);
        }
        if (!game.AuctionSyncEnabled()) {
            if (game.AuctionSyncStartTime() + kSyncWaitMilliseconds <
                LegacyTickMilliseconds()) {
                game.EnableAuctionSync();
            }
            return;
        }

        auto batch = game.AuctionRoom().UnityGoods(existing, mapId);
        for (const MiscNet::CMessage& outgoing : batch.messages) {
            static_cast<void>(outgoing.Send(game.SendQueue(), false));
        }
        if (!batch.error) game.FinishAuctionSyncTurn();
        return;
    }
    case 0x0014'ED06:
    {
        const std::uint32_t playerId =
            std::bit_cast<std::uint32_t>(message.Base().GetLong());
        MiscNet::CMessage response(0x0015'EB07);
        response.Base().Add(playerId);
        if (game.AuctionRoom().AddByteAuctionSelfToClient(response, playerId)) return;
        static_cast<void>(response.Send(game.SendQueue(), false));
        return;
    }
    case 0x0014'ED07:
    {
        const std::uint32_t playerId =
            std::bit_cast<std::uint32_t>(message.Base().GetLong());
        const std::uint32_t operation =
            std::bit_cast<std::uint32_t>(message.Base().GetLong());
        if (game.AuctionRoom().ComputePlayerPage(playerId, operation)) return;
        MiscNet::CMessage response(0x0015'EB08);
        response.Base().Add(playerId);
        if (game.AuctionRoom().AddByteAtPageByTime(playerId, response)) return;
        static_cast<void>(response.Send(game.SendQueue(), false));
        return;
    }
    case 0x0014'ED08:
    {
        const CGUID guid = ReadGuid(message.Base());
        if (!guid.IsInvalided()) {
            static_cast<void>(game.AuctionRoom().PushItemToOptList(guid, 1, 0));
        }
        return;
    }
    case 0x0014'ED09:
    {
        Auction::PlayerOptNode condition;
        condition.playerId = std::bit_cast<std::uint32_t>(message.Base().GetLong());
        condition.lowLevel = message.Base().GetLong();
        condition.upLevel = message.Base().GetLong();
        condition.useSelf = message.Base().GetLong();
        condition.moneyType = message.Base().GetLong();
        condition.weaponType = message.Base().GetLong();
        const auto name = message.Base().GetStrBytes(condition.goodsName.size());
        if (name) {
            std::copy_n(name->begin(), std::min(name->size(), condition.goodsName.size()),
                        condition.goodsName.begin());
        }
        game.AuctionRoom().ModifyPlayerSeachCondition(std::move(condition));
        return;
    }
    default:
        return;
    }
}
}
