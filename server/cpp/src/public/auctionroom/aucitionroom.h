#pragma once

#include "auctionnode.h"
#include "auctionroom.h"
#include "../../nets/netmisc/message.h"

#include <cstdint>
#include <deque>
#include <map>
#include <memory>
#include <optional>
#include <variant>
#include <vector>

namespace Auction
{
enum class AuctionRoomError
{
    MissingGoodsType,
    MissingLevelLimit,
    LegacyStringWithoutTerminator,
    GoodsCountOutsideLegacyRange,
    SerializeBlocked,
    DuplicateTerminalGoods,
    UnsupportedStateDanglingPointer,
};

struct UnityGoodsResult
{
    std::vector<MiscNet::CMessage> messages;
    std::optional<AuctionRoomError> error;
};

/*
 * Исходный владелец: public/auctionroom/aucitionroom.cpp / auctionroom.h.
 *
 * Контейнеры и переходы восстановлены по MiscServer EXE/PDB и проверенной
 * Rust-реконструкции. std::map/deque/vector сохраняют исходный порядок; ручное
 * владение узлами заменено unique_ptr. Важные странности оригинала — всегда
 * false у PushItemToOptList, сохранение opt/search при Clear и одна back-запись
 * за AI — выражены явно, а не исправлены по вкусу Linux-донора.
 */
class CAuctionRoom
{
public:
    void Clear();
    [[nodiscard]] std::optional<AuctionRoomError>
    AddItemToAuctionRoom(std::unique_ptr<CGoodsNode> item,
                         std::uint32_t& deletedNewCount,
                         bool& added);
    [[nodiscard]] const CGoodsNode* QueryGoodsNodeInfo(const CGUID& guid) const;
    [[nodiscard]] std::uint32_t AuctionGoodsCount() const noexcept;

    [[nodiscard]] bool PushItemToOptList(const CGUID& guid,
                                         std::uint32_t operation,
                                         std::int32_t playerId);
    [[nodiscard]] std::optional<AuctionRoomError>
    AddByteAuctionSelfToClient(MiscNet::CMessage& message,
                               std::uint32_t playerId) const;
    void ModifyPlayerSeachCondition(PlayerOptNode condition);
    [[nodiscard]] std::optional<AuctionRoomError>
    ComputePlayerPage(std::uint32_t playerId, std::uint32_t operation);
    [[nodiscard]] std::optional<AuctionRoomError>
    AddByteAtPageByTime(std::uint32_t playerId, MiscNet::CMessage& message) const;
    [[nodiscard]] UnityGoodsResult UnityGoods(
        const std::map<CGUID, bool>& existing,
        std::uint8_t mapId) const;

    [[nodiscard]] std::optional<AuctionRoomError> AI(ClientSendQueue* sender);

private:
    using GuidList = std::vector<CGUID>;
    using GoodsMap = std::map<CGUID, std::unique_ptr<CGoodsNode>>;

    [[nodiscard]] CGoodsNode* QueryGoodsNodeInfo(const CGUID& guid);
    [[nodiscard]] bool AddItemToDelList(const CGUID& guid);
    [[nodiscard]] bool DelItemFromAuctionRoom(const CGUID& guid);
    [[nodiscard]] bool DelItemFromAuctionRoomForSucessed(const CGUID& guid);
    [[nodiscard]] bool DelItemFromAuctionRoomByPreBuy(const CGUID& guid);
    void DoneOptList();
    void DoneAuction();
    [[nodiscard]] std::optional<AuctionRoomError> DoneDelList();
    void DoneSucessedGoodsList(ClientSendQueue* sender);
    void DoneBackList(ClientSendQueue* sender);

    [[nodiscard]] bool DelItemFromTimeList(std::uint32_t ticket, const CGUID& guid);
    [[nodiscard]] bool DelItemFromOwnerList(std::int32_t ownerId, const CGUID& guid);
    [[nodiscard]] bool DelItemFromTypeList(std::uint8_t goodsType, const CGUID& guid);
    [[nodiscard]] std::optional<AuctionRoomError> ClearRecond(const CGUID& guid);
    [[nodiscard]] std::variant<std::unique_ptr<CGoodsNode>, AuctionRoomError>
    PopItemFromGoodsList(const CGUID& guid);
    [[nodiscard]] bool QueryItemFromOwnerList(std::int32_t playerId,
                                              GuidList& items) const;
    [[nodiscard]] std::variant<bool, AuctionRoomError>
    IsMatchCondition(std::uint32_t playerId,
                     const PlayerOptNode& condition,
                     const CGUID& guid) const;

    GoodsMap m_AuctionGoods;
    std::map<std::uint32_t, GuidList> m_TimeTicket;
    std::map<std::int32_t, GuidList> m_OwnerList;
    std::map<std::uint8_t, GuidList> m_TypeList;
    std::deque<GuidList> m_DeleteList;
    GoodsMap m_SucessedGoods;
    GoodsMap m_BackGoods;
    std::map<std::uint32_t, PlayerOptNode> m_PlayerSearch;
    std::map<CGUID, std::uint32_t> m_OptList;
};
}
