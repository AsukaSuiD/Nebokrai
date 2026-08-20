#pragma once

#include "../guid.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <variant>
#include <vector>

namespace Auction
{
inline constexpr std::size_t kLegacyAuctionStringSize = 0x100;
inline constexpr std::size_t kAuctionInfoSize = 0x22C;

enum class GoodsState : std::int32_t
{
    None = 0,
    Auction = 1,
    Sucessed = 2,
    Back = 3,
    Undo = 4,
    PreBuy = 5,
};

enum class GoodsNodeCodecError
{
    UnexpectedEnd,
    LegacyStringOverflow,
    MissingGoodsType,
    MissingLevelLimit,
    LegacyStringWithoutTerminator,
    GoodsLengthOutsideLegacyRange,
};

/*
 * Исходный владелец: public/auctionroom/auctionnode.cpp / .h.
 *
 * Misc/Game/World EXE/PDB и Rust-реконструкция совпадают по 0x478-байтовому
 * старому классу и byte-exact Serialize/UnSerialize. Непрозрачный AuctionInfo
 * сохраняется 0x22C байтами; только подтверждённый buyer id по +0x224 имеет
 * типизированную границу. std::vector заменяет старое ручное владение.
 */
class CGoodsNode
{
public:
    CGoodsNode();

    void Clear();
    [[nodiscard]] std::optional<GoodsNodeCodecError>
    UnSerialize(std::span<const std::uint8_t> source, std::size_t& cursor);
    [[nodiscard]] std::variant<std::vector<std::uint8_t>, GoodsNodeCodecError>
    Serialize() const;

    [[nodiscard]] const CGUID& Guid() const noexcept;
    [[nodiscard]] std::uint32_t AddTicket() const noexcept;
    [[nodiscard]] std::uint32_t OwnerID() const noexcept;
    [[nodiscard]] std::uint8_t MoneyType() const noexcept;
    [[nodiscard]] std::optional<std::uint8_t> GoodsTypeValue() const noexcept;
    [[nodiscard]] std::optional<std::uint32_t> LevelLimit() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t, kLegacyAuctionStringSize>
    GoodsName() const noexcept;
    [[nodiscard]] GoodsState State() const noexcept;
    [[nodiscard]] std::uint32_t BaseIndex() const noexcept;
    [[nodiscard]] std::int32_t Amount() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> GoodsBytes() const noexcept;
    [[nodiscard]] bool HasOfferPrice() const noexcept;
    [[nodiscard]] std::uint32_t AuctionBuyerID() const noexcept;

    void MarkAsAuction() noexcept;
    void MarkAsUndo() noexcept;
    void MarkAsPreBuy() noexcept;
    void MarkAsSucessed() noexcept;
    void MarkAsBack() noexcept;
    void SetAuctionBuyerID(std::uint32_t buyerId) noexcept;

private:
    bool m_Db{};
    std::uint32_t m_AddTicket{};
    std::array<std::uint8_t, kLegacyAuctionStringSize> m_Account{};
    std::uint32_t m_OwnerID{};
    std::uint32_t m_AuctionTime{};
    std::uint8_t m_MoneyType{};
    std::optional<std::uint8_t> m_GoodsType;
    std::int32_t m_NpcPrice{};
    std::int32_t m_Amount{};
    GoodsState m_State{GoodsState::None};
    bool m_OfferPrice{};
    CGUID m_Guid;
    std::optional<std::uint32_t> m_LevelLimit;
    std::array<std::uint8_t, kLegacyAuctionStringSize> m_GoodsName{};
    std::uint32_t m_BaseIndex{};
    std::array<std::uint8_t, kAuctionInfoSize> m_AuctionInfo{};
    std::vector<std::uint8_t> m_GoodsBytes;
};
}
