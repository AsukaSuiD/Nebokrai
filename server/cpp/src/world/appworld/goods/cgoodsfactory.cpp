#include "cgoodsfactory.h"

#include <utility>

bool CGoodsFactory::Register(std::uint32_t id, CGoodsBaseProperties properties)
{
    const std::string originalName = properties.GetOriginalName();
    const auto [position, inserted] = m_Properties.emplace(id, std::move(properties));
    if (!inserted) {
        return false;
    }
    if (!originalName.empty()) {
        m_ByOriginalName.emplace(originalName, id);
    }
    return true;
}

void CGoodsFactory::Clear() noexcept
{
    m_Properties.clear();
    m_ByOriginalName.clear();
    m_GoldCoinIndex = 0;
    m_YuanBaoIndex = 0;
    m_JiFenIndex = 0;
}

const CGoodsBaseProperties* CGoodsFactory::Query(std::uint32_t id) const noexcept
{
    const auto found = m_Properties.find(id);
    return found == m_Properties.end() ? nullptr : &found->second;
}

std::optional<std::uint32_t> CGoodsFactory::QueryByOriginalName(std::string_view name) const
{
    const auto found = m_ByOriginalName.find(name);
    return found == m_ByOriginalName.end() ? std::nullopt : std::optional{found->second};
}

std::unique_ptr<CGoods> CGoodsFactory::Create(std::uint32_t id, bool rollModifiers) const
{
    (void)rollModifiers;
    const CGoodsBaseProperties* properties = Query(id);
    if (properties == nullptr) {
        return {};
    }
    auto goods = std::make_unique<CGoods>();
    goods->SetBasePropertiesIndex(id);
    goods->SetName(properties->GetName());
    goods->SetPrice(properties->GetPrice());
    goods->SetGoodsDescription(properties->GetDescription());
    CGUID guid;
    if (!CGUID::CreateGUID(guid)) {
        return {};
    }
    goods->SetGUID(guid);
    return goods;
}

std::unique_ptr<CGoods> CGoodsFactory::Unserialize(std::span<const std::uint8_t> input,
                                                   std::size_t& offset) const
{
    auto goods = std::make_unique<CGoods>();
    const std::size_t initialOffset = offset;
    if (!goods->DecordFromByteArray(input, offset, true) ||
        Query(goods->GetBasePropertiesIndex()) == nullptr) {
        offset = initialOffset;
        return {};
    }
    return goods;
}

void CGoodsFactory::SetCurrencyIndices(std::uint32_t goldCoin,
                                       std::uint32_t yuanBao,
                                       std::uint32_t jiFen) noexcept
{
    m_GoldCoinIndex = goldCoin;
    m_YuanBaoIndex = yuanBao;
    m_JiFenIndex = jiFen;
}
