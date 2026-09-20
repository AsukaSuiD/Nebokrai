#include "auctionnode.h"

#include <algorithm>
#include <bit>
#include <limits>

namespace Auction
{
namespace
{
constexpr std::size_t kAuctionBuyerIDOffset = 0x224;

class Reader
{
public:
    Reader(std::span<const std::uint8_t> source, std::size_t& cursor)
        : m_Source(source), m_Cursor(cursor)
    {
    }

    bool ReadByte(std::uint8_t& value)
    {
        auto bytes = Take(1);
        if (bytes.empty()) return false;
        value = bytes[0];
        return true;
    }

    bool ReadU32(std::uint32_t& value)
    {
        auto bytes = Take(4);
        if (bytes.size() != 4) return false;
        value = static_cast<std::uint32_t>(bytes[0]) |
                (static_cast<std::uint32_t>(bytes[1]) << 8U) |
                (static_cast<std::uint32_t>(bytes[2]) << 16U) |
                (static_cast<std::uint32_t>(bytes[3]) << 24U);
        return true;
    }

    bool ReadI32(std::int32_t& value)
    {
        std::uint32_t word{};
        if (!ReadU32(word)) return false;
        value = std::bit_cast<std::int32_t>(word);
        return true;
    }

    template <std::size_t Size>
    bool ReadExact(std::array<std::uint8_t, Size>& value)
    {
        auto bytes = Take(Size);
        if (bytes.size() != Size) return false;
        std::copy(bytes.begin(), bytes.end(), value.begin());
        return true;
    }

    template <std::size_t Size>
    std::optional<GoodsNodeCodecError> ReadCString(std::array<std::uint8_t, Size>& value)
    {
        value.fill(0);
        for (std::size_t index = 0; ; ++index) {
            std::uint8_t byte{};
            if (!ReadByte(byte)) return GoodsNodeCodecError::UnexpectedEnd;
            if (index >= value.size()) return GoodsNodeCodecError::LegacyStringOverflow;
            value[index] = byte;
            if (byte == 0) return std::nullopt;
        }
    }

    std::span<const std::uint8_t> Take(std::size_t size)
    {
        if (m_Cursor > m_Source.size() || size > m_Source.size() - m_Cursor) {
            return {};
        }
        auto result = m_Source.subspan(m_Cursor, size);
        m_Cursor += size;
        return result;
    }

private:
    std::span<const std::uint8_t> m_Source;
    std::size_t& m_Cursor;
};

void AppendU32(std::vector<std::uint8_t>& output, std::uint32_t value)
{
    output.push_back(static_cast<std::uint8_t>(value));
    output.push_back(static_cast<std::uint8_t>(value >> 8U));
    output.push_back(static_cast<std::uint8_t>(value >> 16U));
    output.push_back(static_cast<std::uint8_t>(value >> 24U));
}

template <std::size_t Size>
std::optional<std::span<const std::uint8_t>> CString(
    const std::array<std::uint8_t, Size>& value)
{
    const auto terminator = std::find(value.begin(), value.end(), std::uint8_t{0});
    if (terminator == value.end()) return std::nullopt;
    return std::span<const std::uint8_t>(value.data(),
                                        static_cast<std::size_t>(terminator - value.begin()) + 1U);
}
}

CGoodsNode::CGoodsNode() { Clear(); }

void CGoodsNode::Clear()
{
    m_Db = true;
    m_AddTicket = 0;
    m_Account.fill(0);
    m_OwnerID = 0;
    m_AuctionTime = 0;
    m_MoneyType = 0;
    m_NpcPrice = 0;
    m_Amount = 1;
    m_State = GoodsState::None;
    m_OfferPrice = false;
    m_Guid = CGUID::GUID_INVALID;
    m_BaseIndex = 0;
    m_GoodsName.fill(0);
    m_AuctionInfo.fill(0);
    m_GoodsBytes.clear();
}

std::optional<GoodsNodeCodecError>
CGoodsNode::UnSerialize(std::span<const std::uint8_t> source, std::size_t& cursor)
{
    Clear();
    Reader reader(source, cursor);
    std::uint8_t byte{};
    std::int32_t state{};
    if (!reader.ReadByte(byte)) return GoodsNodeCodecError::UnexpectedEnd;
    m_Db = byte != 0;
    if (!reader.ReadU32(m_AddTicket)) return GoodsNodeCodecError::UnexpectedEnd;
    if (auto error = reader.ReadCString(m_Account)) return error;
    if (!reader.ReadU32(m_OwnerID) || !reader.ReadU32(m_AuctionTime) ||
        !reader.ReadByte(m_MoneyType) || !reader.ReadByte(byte)) {
        return GoodsNodeCodecError::UnexpectedEnd;
    }
    m_GoodsType = byte;
    if (!reader.ReadI32(m_NpcPrice) || !reader.ReadI32(m_Amount) ||
        !reader.ReadI32(state) || !reader.ReadByte(byte)) {
        return GoodsNodeCodecError::UnexpectedEnd;
    }
    m_State = static_cast<GoodsState>(state);
    m_OfferPrice = byte != 0;
    if (!reader.ReadByte(byte)) return GoodsNodeCodecError::UnexpectedEnd;
    if (byte == 0) {
        m_Guid = CGUID::GUID_INVALID;
    } else {
        std::array<std::uint8_t, 16> guid{};
        if (!reader.ReadExact(guid)) return GoodsNodeCodecError::UnexpectedEnd;
        m_Guid = CGUID::FromLegacyBytes(guid);
    }
    std::uint32_t level{};
    if (!reader.ReadU32(m_BaseIndex) || !reader.ReadU32(level)) {
        return GoodsNodeCodecError::UnexpectedEnd;
    }
    m_LevelLimit = level;
    if (auto error = reader.ReadCString(m_GoodsName)) return error;
    if (!reader.ReadExact(m_AuctionInfo)) return GoodsNodeCodecError::UnexpectedEnd;
    std::uint32_t goodsLength{};
    if (!reader.ReadU32(goodsLength)) return GoodsNodeCodecError::UnexpectedEnd;
    auto goods = reader.Take(goodsLength);
    if (goods.size() != goodsLength) return GoodsNodeCodecError::UnexpectedEnd;
    m_GoodsBytes.assign(goods.begin(), goods.end());
    return std::nullopt;
}

std::variant<std::vector<std::uint8_t>, GoodsNodeCodecError>
CGoodsNode::Serialize() const
{
    if (!m_GoodsType) return GoodsNodeCodecError::MissingGoodsType;
    if (!m_LevelLimit) return GoodsNodeCodecError::MissingLevelLimit;
    const auto account = CString(m_Account);
    const auto goodsName = CString(m_GoodsName);
    if (!account || !goodsName) return GoodsNodeCodecError::LegacyStringWithoutTerminator;
    if (m_GoodsBytes.size() > std::numeric_limits<std::uint32_t>::max()) {
        return GoodsNodeCodecError::GoodsLengthOutsideLegacyRange;
    }

    std::vector<std::uint8_t> output;
    output.push_back(static_cast<std::uint8_t>(m_Db));
    AppendU32(output, m_AddTicket);
    output.insert(output.end(), account->begin(), account->end());
    AppendU32(output, m_OwnerID);
    AppendU32(output, m_AuctionTime);
    output.push_back(m_MoneyType);
    output.push_back(*m_GoodsType);
    AppendU32(output, std::bit_cast<std::uint32_t>(m_NpcPrice));
    AppendU32(output, std::bit_cast<std::uint32_t>(m_Amount));
    AppendU32(output, std::bit_cast<std::uint32_t>(static_cast<std::int32_t>(m_State)));
    output.push_back(static_cast<std::uint8_t>(m_OfferPrice));
    if (m_Guid.IsInvalided()) {
        output.push_back(0);
    } else {
        output.push_back(0x10);
        const auto bytes = m_Guid.Bytes();
        output.insert(output.end(), bytes.begin(), bytes.end());
    }
    AppendU32(output, m_BaseIndex);
    AppendU32(output, *m_LevelLimit);
    output.insert(output.end(), goodsName->begin(), goodsName->end());
    output.insert(output.end(), m_AuctionInfo.begin(), m_AuctionInfo.end());
    AppendU32(output, static_cast<std::uint32_t>(m_GoodsBytes.size()));
    output.insert(output.end(), m_GoodsBytes.begin(), m_GoodsBytes.end());
    return output;
}

const CGUID& CGoodsNode::Guid() const noexcept { return m_Guid; }
std::uint32_t CGoodsNode::AddTicket() const noexcept { return m_AddTicket; }
std::uint32_t CGoodsNode::OwnerID() const noexcept { return m_OwnerID; }
std::uint8_t CGoodsNode::MoneyType() const noexcept { return m_MoneyType; }
std::optional<std::uint8_t> CGoodsNode::GoodsTypeValue() const noexcept { return m_GoodsType; }
std::optional<std::uint32_t> CGoodsNode::LevelLimit() const noexcept { return m_LevelLimit; }
std::span<const std::uint8_t, kLegacyAuctionStringSize> CGoodsNode::GoodsName() const noexcept
{
    return m_GoodsName;
}
GoodsState CGoodsNode::State() const noexcept { return m_State; }
std::uint32_t CGoodsNode::BaseIndex() const noexcept { return m_BaseIndex; }
std::int32_t CGoodsNode::Amount() const noexcept { return m_Amount; }
std::span<const std::uint8_t> CGoodsNode::GoodsBytes() const noexcept { return m_GoodsBytes; }
bool CGoodsNode::HasOfferPrice() const noexcept { return m_OfferPrice; }

std::uint32_t CGoodsNode::AuctionBuyerID() const noexcept
{
    const auto* bytes = m_AuctionInfo.data() + kAuctionBuyerIDOffset;
    return static_cast<std::uint32_t>(bytes[0]) |
           (static_cast<std::uint32_t>(bytes[1]) << 8U) |
           (static_cast<std::uint32_t>(bytes[2]) << 16U) |
           (static_cast<std::uint32_t>(bytes[3]) << 24U);
}

void CGoodsNode::MarkAsAuction() noexcept { m_State = GoodsState::Auction; }
void CGoodsNode::MarkAsUndo() noexcept { m_State = GoodsState::Undo; }
void CGoodsNode::MarkAsPreBuy() noexcept { m_State = GoodsState::PreBuy; }
void CGoodsNode::MarkAsSucessed() noexcept { m_State = GoodsState::Sucessed; }
void CGoodsNode::MarkAsBack() noexcept { m_State = GoodsState::Back; }

void CGoodsNode::SetAuctionBuyerID(std::uint32_t buyerId) noexcept
{
    m_AuctionInfo[kAuctionBuyerIDOffset] = static_cast<std::uint8_t>(buyerId);
    m_AuctionInfo[kAuctionBuyerIDOffset + 1U] = static_cast<std::uint8_t>(buyerId >> 8U);
    m_AuctionInfo[kAuctionBuyerIDOffset + 2U] = static_cast<std::uint8_t>(buyerId >> 16U);
    m_AuctionInfo[kAuctionBuyerIDOffset + 3U] = static_cast<std::uint8_t>(buyerId >> 24U);
}
}
