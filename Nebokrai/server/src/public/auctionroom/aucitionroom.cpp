#include "aucitionroom.h"

#include <algorithm>
#include <bit>
#include <chrono>
#include <cstring>
#include <limits>

namespace Auction
{
namespace
{
constexpr std::size_t kPageSize = 7;
constexpr std::size_t kOwnerListLimit = 100;
constexpr std::size_t kUnityDetailLimit = 20;

std::uint32_t RealtimeSeconds() noexcept
{
    return static_cast<std::uint32_t>(
        std::chrono::duration_cast<std::chrono::seconds>(
            std::chrono::system_clock::now().time_since_epoch()).count());
}

bool RemoveFirst(std::vector<CGUID>& list, const CGUID& guid)
{
    const auto found = std::find(list.begin(), list.end(), guid);
    if (found == list.end()) return false;
    list.erase(found);
    return true;
}

void AddSerialized(MiscNet::CMessage& message, const std::vector<std::uint8_t>& bytes)
{
    message.Base().Add(bytes.data(), static_cast<std::int32_t>(bytes.size()));
}

std::optional<AuctionRoomError> MapCodecError(GoodsNodeCodecError error)
{
    switch (error) {
    case GoodsNodeCodecError::MissingGoodsType:
        return AuctionRoomError::MissingGoodsType;
    case GoodsNodeCodecError::MissingLevelLimit:
        return AuctionRoomError::MissingLevelLimit;
    case GoodsNodeCodecError::LegacyStringWithoutTerminator:
        return AuctionRoomError::LegacyStringWithoutTerminator;
    default:
        return AuctionRoomError::SerializeBlocked;
    }
}

std::variant<std::span<const std::uint8_t>, AuctionRoomError>
LegacyName(std::span<const std::uint8_t, kLegacyAuctionStringSize> name)
{
    const auto terminator = std::find(name.begin(), name.end(), std::uint8_t{0});
    if (terminator == name.end()) {
        return AuctionRoomError::LegacyStringWithoutTerminator;
    }
    return std::span<const std::uint8_t>(name.data(),
                                        static_cast<std::size_t>(terminator - name.begin()));
}
}

void CAuctionRoom::Clear()
{
    // Исходный Clear намеренно не трогал m_OptList и m_PlayerSearch.
    m_AuctionGoods.clear();
    m_DeleteList.clear();
    m_SucessedGoods.clear();
    m_BackGoods.clear();
    m_TimeTicket.clear();
    m_OwnerList.clear();
    m_TypeList.clear();
}

std::optional<AuctionRoomError>
CAuctionRoom::AddItemToAuctionRoom(std::unique_ptr<CGoodsNode> item,
                                   std::uint32_t& deletedNewCount,
                                   bool& added)
{
    added = false;
    if (!item || item->Guid().IsInvalided() || m_AuctionGoods.contains(item->Guid())) {
        ++deletedNewCount;
        return std::nullopt;
    }

    const CGUID guid = item->Guid();
    m_AuctionGoods.emplace(guid, std::move(item));
    CGoodsNode& stored = *m_AuctionGoods.at(guid);
    stored.MarkAsAuction();
    m_TimeTicket[stored.AddTicket()].push_back(guid);
    m_OwnerList[std::bit_cast<std::int32_t>(stored.OwnerID())].push_back(guid);
    const auto goodsType = stored.GoodsTypeValue();
    if (!goodsType) return AuctionRoomError::MissingGoodsType;
    m_TypeList[*goodsType].push_back(guid);
    added = true;
    return std::nullopt;
}

CGoodsNode* CAuctionRoom::QueryGoodsNodeInfo(const CGUID& guid)
{
    if (guid.IsInvalided()) return nullptr;
    const auto found = m_AuctionGoods.find(guid);
    return found == m_AuctionGoods.end() ? nullptr : found->second.get();
}

const CGoodsNode* CAuctionRoom::QueryGoodsNodeInfo(const CGUID& guid) const
{
    if (guid.IsInvalided()) return nullptr;
    const auto found = m_AuctionGoods.find(guid);
    return found == m_AuctionGoods.end() ? nullptr : found->second.get();
}

std::uint32_t CAuctionRoom::AuctionGoodsCount() const noexcept
{
    return static_cast<std::uint32_t>(m_AuctionGoods.size());
}

bool CAuctionRoom::PushItemToOptList(const CGUID& guid,
                                     std::uint32_t operation,
                                     std::int32_t playerId)
{
    if (m_OptList.contains(guid)) return false;
    CGoodsNode* item = QueryGoodsNodeInfo(guid);
    if (item == nullptr || item->AuctionBuyerID() != 0) return false;
    item->SetAuctionBuyerID(operation == 1 ? item->OwnerID()
                                           : std::bit_cast<std::uint32_t>(playerId));
    m_OptList.emplace(guid, operation);
    // Exact Misc EXE завершает и успешный путь `xor al, al`.
    return false;
}

bool CAuctionRoom::AddItemToDelList(const CGUID& guid)
{
    if (guid.IsInvalided()) return false;
    if (m_DeleteList.empty()) m_DeleteList.emplace_back();
    m_DeleteList.front().push_back(guid);
    return true;
}

bool CAuctionRoom::DelItemFromAuctionRoom(const CGUID& guid)
{
    CGoodsNode* item = QueryGoodsNodeInfo(guid);
    if (item == nullptr || item->State() != GoodsState::Auction) return false;
    item->MarkAsUndo();
    static_cast<void>(AddItemToDelList(guid));
    return true;
}

bool CAuctionRoom::DelItemFromAuctionRoomForSucessed(const CGUID& guid)
{
    CGoodsNode* item = QueryGoodsNodeInfo(guid);
    if (item == nullptr || item->State() != GoodsState::Auction) return false;
    item->MarkAsSucessed();
    static_cast<void>(AddItemToDelList(guid));
    return true;
}

bool CAuctionRoom::DelItemFromAuctionRoomByPreBuy(const CGUID& guid)
{
    CGoodsNode* item = QueryGoodsNodeInfo(guid);
    if (item == nullptr || item->State() != GoodsState::Auction) return false;
    item->MarkAsPreBuy();
    static_cast<void>(AddItemToDelList(guid));
    return true;
}

void CAuctionRoom::DoneOptList()
{
    while (!m_OptList.empty()) {
        const auto [guid, operation] = *m_OptList.begin();
        switch (operation) {
        case 1: static_cast<void>(DelItemFromAuctionRoom(guid)); break;
        case 2: static_cast<void>(DelItemFromAuctionRoomForSucessed(guid)); break;
        case 3: static_cast<void>(DelItemFromAuctionRoomByPreBuy(guid)); break;
        default: break;
        }
        m_OptList.erase(m_OptList.begin());
    }
}

void CAuctionRoom::DoneAuction()
{
    DoneOptList();
    const std::uint32_t now = RealtimeSeconds();
    while (!m_TimeTicket.empty() && m_TimeTicket.begin()->first <= now) {
        m_DeleteList.push_back(std::move(m_TimeTicket.begin()->second));
        m_TimeTicket.erase(m_TimeTicket.begin());
    }
}

bool CAuctionRoom::DelItemFromTimeList(std::uint32_t ticket, const CGUID& guid)
{
    const auto found = m_TimeTicket.find(ticket);
    if (found == m_TimeTicket.end()) return false;
    const bool removed = RemoveFirst(found->second, guid);
    if (found->second.empty()) m_TimeTicket.erase(found);
    return removed;
}

bool CAuctionRoom::DelItemFromOwnerList(std::int32_t ownerId, const CGUID& guid)
{
    const auto found = m_OwnerList.find(ownerId);
    if (found == m_OwnerList.end()) return false;
    const bool removed = RemoveFirst(found->second, guid);
    if (found->second.empty()) m_OwnerList.erase(found);
    return removed;
}

bool CAuctionRoom::DelItemFromTypeList(std::uint8_t goodsType, const CGUID& guid)
{
    const auto found = m_TypeList.find(goodsType);
    if (found == m_TypeList.end()) return false;
    static_cast<void>(RemoveFirst(found->second, guid));
    // В отличие от time/owner исходный helper сохранял пустой type-key.
    return true;
}

std::optional<AuctionRoomError> CAuctionRoom::ClearRecond(const CGUID& guid)
{
    const CGoodsNode* item = QueryGoodsNodeInfo(guid);
    if (item == nullptr) return std::nullopt;
    const auto ticket = item->AddTicket();
    const auto owner = std::bit_cast<std::int32_t>(item->OwnerID());
    const auto goodsType = item->GoodsTypeValue();
    static_cast<void>(DelItemFromTimeList(ticket, guid));
    static_cast<void>(DelItemFromOwnerList(owner, guid));
    if (!goodsType) return AuctionRoomError::MissingGoodsType;
    static_cast<void>(DelItemFromTypeList(*goodsType, guid));
    return std::nullopt;
}

std::variant<std::unique_ptr<CGoodsNode>, AuctionRoomError>
CAuctionRoom::PopItemFromGoodsList(const CGUID& guid)
{
    if (guid.IsInvalided() || !m_AuctionGoods.contains(guid)) {
        return std::unique_ptr<CGoodsNode>{};
    }
    if (auto error = ClearRecond(guid)) return *error;
    auto node = std::move(m_AuctionGoods.at(guid));
    m_AuctionGoods.erase(guid);
    return node;
}

std::optional<AuctionRoomError> CAuctionRoom::DoneDelList()
{
    while (true) {
        while (!m_DeleteList.empty() && m_DeleteList.front().empty()) {
            m_DeleteList.pop_front();
        }
        if (m_DeleteList.empty()) return std::nullopt;

        const CGUID guid = m_DeleteList.front().front();
        CGoodsNode* current = QueryGoodsNodeInfo(guid);
        if (current == nullptr) {
            m_DeleteList.front().erase(m_DeleteList.front().begin());
            continue;
        }
        const GoodsState state = current->State();
        const bool offerPrice = current->HasOfferPrice();

        if (state != GoodsState::Auction && state != GoodsState::Sucessed &&
            state != GoodsState::Undo && state != GoodsState::PreBuy) {
            const auto error = ClearRecond(guid);
            m_DeleteList.front().erase(m_DeleteList.front().begin());
            return error.value_or(AuctionRoomError::UnsupportedStateDanglingPointer);
        }

        auto removed = PopItemFromGoodsList(guid);
        if (const auto* error = std::get_if<AuctionRoomError>(&removed)) return *error;
        auto item = std::move(std::get<std::unique_ptr<CGoodsNode>>(removed));
        if (item) {
            GoodsMap* destination = nullptr;
            if (state == GoodsState::Auction) {
                if (offerPrice) {
                    item->MarkAsSucessed();
                    destination = &m_SucessedGoods;
                } else {
                    item->MarkAsBack();
                    destination = &m_BackGoods;
                }
            } else if (state == GoodsState::Sucessed) {
                destination = &m_SucessedGoods;
            } else {
                destination = &m_BackGoods;
            }

            if (destination->contains(guid)) {
                m_DeleteList.front().erase(m_DeleteList.front().begin());
                return AuctionRoomError::DuplicateTerminalGoods;
            }
            destination->emplace(guid, std::move(item));
        }
        m_DeleteList.front().erase(m_DeleteList.front().begin());
    }
}

void CAuctionRoom::DoneSucessedGoodsList(ClientSendQueue* sender)
{
    while (!m_SucessedGoods.empty()) {
        auto item = m_SucessedGoods.begin();
        const auto serialized = item->second->Serialize();
        if (const auto* bytes = std::get_if<std::vector<std::uint8_t>>(&serialized)) {
            MiscNet::CMessage message(0x0015'EB01);
            AddSerialized(message, *bytes);
            static_cast<void>(message.Send(sender, false));
        }
        m_SucessedGoods.erase(item);
    }
}

void CAuctionRoom::DoneBackList(ClientSendQueue* sender)
{
    if (m_BackGoods.empty()) return;
    auto item = m_BackGoods.begin();
    const auto serialized = item->second->Serialize();
    if (const auto* bytes = std::get_if<std::vector<std::uint8_t>>(&serialized)) {
        MiscNet::CMessage message(0x0015'EB02);
        AddSerialized(message, *bytes);
        static_cast<void>(message.Send(sender, false));
    }
    m_BackGoods.erase(item);
}

std::optional<AuctionRoomError> CAuctionRoom::AI(ClientSendQueue* sender)
{
    DoneAuction();
    const auto error = DoneDelList();
    DoneSucessedGoodsList(sender);
    DoneBackList(sender);
    return error;
}

bool CAuctionRoom::QueryItemFromOwnerList(std::int32_t playerId,
                                          GuidList& items) const
{
    const auto found = m_OwnerList.find(playerId);
    if (found != m_OwnerList.end()) {
        for (const CGUID& guid : found->second) {
            items.push_back(guid);
            if (items.size() > kOwnerListLimit - 1U) return true;
        }
    }
    return !items.empty();
}

std::optional<AuctionRoomError>
CAuctionRoom::AddByteAuctionSelfToClient(MiscNet::CMessage& message,
                                         std::uint32_t playerId) const
{
    GuidList items;
    static_cast<void>(QueryItemFromOwnerList(std::bit_cast<std::int32_t>(playerId), items));
    message.Base().Add(static_cast<std::int32_t>(items.size()));
    for (const CGUID& guid : items) {
        const CGoodsNode* item = QueryGoodsNodeInfo(guid);
        if (item == nullptr) {
            message.Base().Add(std::int32_t{0});
            continue;
        }
        message.Base().Add(std::int32_t{1});
        const auto serialized = item->Serialize();
        if (const auto* error = std::get_if<GoodsNodeCodecError>(&serialized)) {
            return MapCodecError(*error);
        }
        AddSerialized(message, std::get<std::vector<std::uint8_t>>(serialized));
    }
    return std::nullopt;
}

void CAuctionRoom::ModifyPlayerSeachCondition(PlayerOptNode condition)
{
    m_PlayerSearch.insert_or_assign(condition.playerId, std::move(condition));
}

std::optional<AuctionRoomError>
CAuctionRoom::ComputePlayerPage(std::uint32_t playerId, std::uint32_t operation)
{
    if (m_AuctionGoods.size() > std::numeric_limits<std::uint32_t>::max()) {
        return AuctionRoomError::GoodsCountOutsideLegacyRange;
    }
    const std::uint32_t pageCount =
        (static_cast<std::uint32_t>(m_AuctionGoods.size()) + 6U) / 7U;
    const auto found = m_PlayerSearch.find(playerId);
    if (found == m_PlayerSearch.end()) return std::nullopt;

    auto& page = found->second.currentPage;
    if (operation == 1) {
        const std::uint32_t next = page + 1U;
        page = pageCount <= next ? pageCount - 1U : next;
    } else if (operation == 2) {
        if (page != 0) --page;
    } else {
        page = 0;
    }
    return std::nullopt;
}

std::variant<bool, AuctionRoomError>
CAuctionRoom::IsMatchCondition(std::uint32_t playerId,
                               const PlayerOptNode& condition,
                               const CGUID& guid) const
{
    const CGoodsNode* item = QueryGoodsNodeInfo(guid);
    if (item == nullptr || playerId == item->OwnerID()) return false;

    auto conditionName = LegacyName(condition.goodsName);
    auto itemName = LegacyName(item->GoodsName());
    if (const auto* error = std::get_if<AuctionRoomError>(&conditionName)) return *error;
    if (const auto* error = std::get_if<AuctionRoomError>(&itemName)) return *error;
    const auto needle = std::get<std::span<const std::uint8_t>>(conditionName);
    const auto haystack = std::get<std::span<const std::uint8_t>>(itemName);
    const bool nameMatches = needle.empty() ||
        std::search(haystack.begin(), haystack.end(), needle.begin(), needle.end()) !=
            haystack.end();

    const auto level = item->LevelLimit();
    const auto goodsType = item->GoodsTypeValue();
    if (!level) return AuctionRoomError::MissingLevelLimit;
    if (!goodsType) return AuctionRoomError::MissingGoodsType;
    const bool levelMatches = static_cast<std::uint32_t>(condition.lowLevel) <= *level &&
                              *level <= static_cast<std::uint32_t>(condition.upLevel);
    const bool moneyMatches = static_cast<std::uint32_t>(item->MoneyType()) ==
                              static_cast<std::uint32_t>(condition.moneyType);
    const bool weaponMatches = condition.weaponType == -1 ||
        static_cast<std::uint32_t>(*goodsType) ==
            static_cast<std::uint32_t>(condition.weaponType);
    return nameMatches && levelMatches && moneyMatches && weaponMatches;
}

std::optional<AuctionRoomError>
CAuctionRoom::AddByteAtPageByTime(std::uint32_t playerId,
                                  MiscNet::CMessage& message) const
{
    const auto conditionIt = m_PlayerSearch.find(playerId);
    if (conditionIt == m_PlayerSearch.end()) return std::nullopt;
    if (m_AuctionGoods.size() > std::numeric_limits<std::uint32_t>::max()) {
        return AuctionRoomError::GoodsCountOutsideLegacyRange;
    }
    const PlayerOptNode& condition = conditionIt->second;
    std::int32_t remainingToSkip = std::bit_cast<std::int32_t>(
        condition.currentPage * static_cast<std::uint32_t>(kPageSize));
    std::uint32_t visited{};
    auto startTicket = m_TimeTicket.begin();
    std::size_t skipInStart{};

    if (remainingToSkip > 0) {
        startTicket = m_TimeTicket.end();
        for (auto ticket = m_TimeTicket.begin(); ticket != m_TimeTicket.end(); ++ticket) {
            std::size_t visitedInTicket{};
            for (const CGUID& guid : ticket->second) {
                ++visitedInTicket;
                ++visited;
                const auto matches = IsMatchCondition(playerId, condition, guid);
                if (const auto* error = std::get_if<AuctionRoomError>(&matches)) {
                    return *error;
                }
                if (std::get<bool>(matches) && --remainingToSkip == 0) {
                    startTicket = ticket;
                    skipInStart = visitedInTicket;
                    break;
                }
            }
            if (startTicket != m_TimeTicket.end()) break;
        }
    }

    if (startTicket == m_TimeTicket.end()) {
        message.Base().Add(std::int32_t{0});
        return std::nullopt;
    }

    message.Base().Add(static_cast<std::uint32_t>(m_AuctionGoods.size()) - visited);
    message.Base().Add(static_cast<std::int32_t>(kPageSize));
    std::size_t slots = kPageSize;
    bool firstTicket = true;
    for (auto ticket = startTicket; ticket != m_TimeTicket.end(); ++ticket) {
        const std::size_t skip = firstTicket ? skipInStart : 0U;
        firstTicket = false;
        for (auto guid = ticket->second.begin() +
                         static_cast<std::ptrdiff_t>(std::min(skip, ticket->second.size()));
             guid != ticket->second.end(); ++guid) {
            if (slots == 0) return std::nullopt;
            const auto matches = IsMatchCondition(playerId, condition, *guid);
            if (const auto* error = std::get_if<AuctionRoomError>(&matches)) return *error;
            if (!std::get<bool>(matches)) continue;
            const CGoodsNode* item = QueryGoodsNodeInfo(*guid);
            message.Base().Add(std::int32_t{1});
            const auto serialized = item->Serialize();
            if (const auto* error = std::get_if<GoodsNodeCodecError>(&serialized)) {
                return MapCodecError(*error);
            }
            AddSerialized(message, std::get<std::vector<std::uint8_t>>(serialized));
            --slots;
        }
    }
    return std::nullopt;
}

UnityGoodsResult CAuctionRoom::UnityGoods(const std::map<CGUID, bool>& existing,
                                          std::uint8_t mapId) const
{
    MiscNet::CMessage unity(0x0015'EB04);
    unity.Base().Add(mapId);
    UnityGoodsResult result;
    std::size_t detailSlots = kUnityDetailLimit;
    bool needSend = false;

    if (!m_AuctionGoods.empty()) {
        auto input = existing.begin();
        auto primary = m_AuctionGoods.begin();
        while (input != existing.end()) {
            if (primary == m_AuctionGoods.end()) break;
            if (input->first == primary->first) {
                unity.Base().Add(std::uint8_t{1});
                unity.Base().Add(primary->first);
                ++input;
                ++primary;
            } else if (input->first < primary->first) {
                needSend = true;
                ++input;
            } else {
                unity.Base().Add(std::uint8_t{1});
                unity.Base().Add(primary->first);
                if (detailSlots != 0) {
                    --detailSlots;
                    needSend = true;
                    const auto serialized = primary->second->Serialize();
                    if (const auto* error = std::get_if<GoodsNodeCodecError>(&serialized)) {
                        result.error = MapCodecError(*error);
                        return result;
                    }
                    MiscNet::CMessage detail(0x0015'EB03);
                    detail.Base().Add(mapId);
                    AddSerialized(detail, std::get<std::vector<std::uint8_t>>(serialized));
                    result.messages.push_back(std::move(detail));
                }
                ++primary;
            }
        }
        for (; primary != m_AuctionGoods.end(); ++primary) {
            unity.Base().Add(std::uint8_t{1});
            unity.Base().Add(primary->first);
            if (detailSlots != 0) {
                --detailSlots;
                needSend = true;
                const auto serialized = primary->second->Serialize();
                if (const auto* error = std::get_if<GoodsNodeCodecError>(&serialized)) {
                    result.error = MapCodecError(*error);
                    return result;
                }
                MiscNet::CMessage detail(0x0015'EB03);
                detail.Base().Add(mapId);
                AddSerialized(detail, std::get<std::vector<std::uint8_t>>(serialized));
                result.messages.push_back(std::move(detail));
            }
        }
        if (!needSend && input == existing.end()) return result;
    }
    result.messages.push_back(std::move(unity));
    return result;
}
}
