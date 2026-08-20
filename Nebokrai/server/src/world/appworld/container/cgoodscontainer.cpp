#include "cgoodscontainer.h"

#include <algorithm>
#include <cstring>
#include <limits>

namespace
{
void AppendCount(std::vector<std::uint8_t>& output, std::uint32_t count)
{
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&count);
    output.insert(output.end(), bytes, bytes + sizeof(count));
}

bool ReadCount(std::span<const std::uint8_t> input, std::size_t& offset, std::uint32_t& count)
{
    if (offset > input.size() || input.size() - offset < sizeof(count)) {
        return false;
    }
    std::memcpy(&count, input.data() + offset, sizeof(count));
    offset += sizeof(count);
    return true;
}
}

void CGoodsContainer::Clear(void* context)
{
    while (!m_Goods.empty()) {
        auto node = m_Goods.extract(m_Goods.begin());
        NotifyRemoved(*node.mapped(), context);
    }
}

void CGoodsContainer::Release(void* context)
{
    Clear(context);
    m_OwnerType = 0;
    m_OwnerId = 0;
}

bool CGoodsContainer::Add(GoodsPtr& goods, void* context)
{
    if (!goods || IsFull()) {
        return false;
    }
    const CGUID guid = goods->GetGUID();
    if (m_Goods.contains(guid)) {
        return false;
    }
    CGoods* added = goods.get();
    m_Goods.emplace(guid, std::move(goods));
    NotifyAdded(*added, context);
    return true;
}

bool CGoodsContainer::AddFromDB(GoodsPtr& goods, std::uint32_t position, void* context)
{
    (void)position;
    return Add(goods, context);
}

CGoodsContainer::GoodsPtr CGoodsContainer::Remove(const CGUID& guid, void* context)
{
    auto node = m_Goods.extract(guid);
    if (node.empty()) {
        return {};
    }
    NotifyRemoved(*node.mapped(), context);
    return std::move(node.mapped());
}

CGoodsContainer::GoodsPtr CGoodsContainer::RemoveAt(std::uint32_t position,
                                                     std::uint32_t amount,
                                                     void* context)
{
    CGoods* goods = GetGoods(position);
    if (goods == nullptr || amount == 0 || amount > goods->GetAmount()) {
        return {};
    }
    if (amount == goods->GetAmount()) {
        return Remove(goods->GetGUID(), context);
    }

    auto split = Factory().Create(goods->GetBasePropertiesIndex(), false);
    if (!split) {
        return {};
    }
    split->SetAmount(amount);
    goods->SetAmount(goods->GetAmount() - amount);
    return split;
}

bool CGoodsContainer::Serialize(std::vector<std::uint8_t>& output, bool includeChild) const
{
    if (m_Goods.size() > std::numeric_limits<std::uint32_t>::max()) {
        return false;
    }
    AppendCount(output, static_cast<std::uint32_t>(m_Goods.size()));
    for (const auto& [guid, goods] : m_Goods) {
        (void)guid;
        if (Factory().Query(goods->GetBasePropertiesIndex()) == nullptr ||
            !goods->AddToByteArray(output, includeChild)) {
            return false;
        }
    }
    return true;
}

bool CGoodsContainer::Unserialize(std::span<const std::uint8_t> input, std::size_t& offset)
{
    Clear();
    std::uint32_t count{};
    if (!ReadCount(input, offset, count)) {
        return false;
    }
    for (std::uint32_t index = 0; index < count; ++index) {
        GoodsPtr goods = Factory().Unserialize(input, offset);
        if (!goods || !Add(goods)) {
            return false;
        }
    }
    return true;
}

void CGoodsContainer::Traverse(CContainerListener& listener)
{
    for (auto& [guid, goods] : m_Goods) {
        (void)guid;
        listener.OnTraversingContainer(*this, *goods);
    }
}

void CGoodsContainer::AI()
{
    for (auto& [guid, goods] : m_Goods) {
        (void)guid;
        goods->AI();
    }
}

std::uint32_t CGoodsContainer::GetGoodsAmount() const noexcept
{
    return static_cast<std::uint32_t>(m_Goods.size());
}

CGoods* CGoodsContainer::GetGoods(std::uint32_t position) const noexcept
{
    if (position >= m_Goods.size()) {
        return nullptr;
    }
    auto found = m_Goods.begin();
    std::advance(found, position);
    return found->second.get();
}

CGoods* CGoodsContainer::Find(const CGUID& guid) const noexcept
{
    const auto found = m_Goods.find(guid);
    return found == m_Goods.end() ? nullptr : found->second.get();
}

bool CGoodsContainer::QueryGoodsPosition(const CGUID& guid,
                                         std::uint32_t& position) const noexcept
{
    const auto found = m_Goods.find(guid);
    if (found == m_Goods.end()) {
        return false;
    }
    position = static_cast<std::uint32_t>(std::distance(m_Goods.begin(), found));
    return true;
}

bool CGoodsContainer::IsGoodsExisted(std::uint32_t basePropertiesIndex) const noexcept
{
    return GetFirstGoods(basePropertiesIndex) != nullptr;
}

CGoods* CGoodsContainer::GetFirstGoods(std::uint32_t basePropertiesIndex) const noexcept
{
    const auto found = std::ranges::find_if(m_Goods, [basePropertiesIndex](const auto& item) {
        return item.second->GetBasePropertiesIndex() == basePropertiesIndex;
    });
    return found == m_Goods.end() ? nullptr : found->second.get();
}

void CGoodsContainer::GetGoods(std::uint32_t basePropertiesIndex,
                               std::vector<CGoods*>& output) const
{
    for (const auto& [guid, goods] : m_Goods) {
        (void)guid;
        if (goods->GetBasePropertiesIndex() == basePropertiesIndex) {
            output.push_back(goods.get());
        }
    }
}

void CGoodsContainer::SetOwner(std::int32_t type, std::int32_t id) noexcept
{
    m_OwnerType = type;
    m_OwnerId = id;
}
