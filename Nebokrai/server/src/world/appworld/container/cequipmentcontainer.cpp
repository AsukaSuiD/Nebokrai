#include "cequipmentcontainer.h"

#include <cstring>
#include <limits>

namespace
{
void AppendU32(std::vector<std::uint8_t>& output, std::uint32_t value)
{
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), bytes, bytes + sizeof(value));
}

bool ReadU32(std::span<const std::uint8_t> input, std::size_t& offset, std::uint32_t& value)
{
    if (offset > input.size() || input.size() - offset < sizeof(value)) {
        return false;
    }
    std::memcpy(&value, input.data() + offset, sizeof(value));
    offset += sizeof(value);
    return true;
}
}

bool CEquipmentContainer::Add(Column column,
                              std::unique_ptr<CGoods>& goods,
                              void* context)
{
    if (!goods || m_Equipment.contains(column)) {
        return false;
    }
    const CGoodsBaseProperties* properties =
        m_Factory.Query(goods->GetBasePropertiesIndex());
    if (properties == nullptr ||
        properties->GetGoodsType() != CGoodsBaseProperties::GoodsType::Equipment ||
        !Accepts(column, properties->GetEquipPlace())) {
        return false;
    }
    CGoods* added = goods.get();
    m_Equipment.emplace(column, std::move(goods));
    NotifyAdded(*added, context);
    return true;
}

bool CEquipmentContainer::AddFromDB(std::uint32_t position,
                                    std::unique_ptr<CGoods>& goods,
                                    void* context)
{
    return IsColumn(position) && Add(static_cast<Column>(position), goods, context);
}

std::unique_ptr<CGoods> CEquipmentContainer::Remove(const CGUID& guid, void* context)
{
    for (auto found = m_Equipment.begin(); found != m_Equipment.end(); ++found) {
        if (found->second->GetGUID() == guid) {
            auto goods = std::move(found->second);
            m_Equipment.erase(found);
            NotifyRemoved(*goods, context);
            return goods;
        }
    }
    return {};
}

void CEquipmentContainer::Clear(void* context)
{
    while (!m_Equipment.empty()) {
        auto node = m_Equipment.extract(m_Equipment.begin());
        NotifyRemoved(*node.mapped(), context);
    }
}

void CEquipmentContainer::Release(void* context)
{
    Clear(context);
}

CGoods* CEquipmentContainer::GetGoods(Column column) const noexcept
{
    const auto found = m_Equipment.find(column);
    return found == m_Equipment.end() ? nullptr : found->second.get();
}

CGoods* CEquipmentContainer::GetGoods(std::uint32_t position) const noexcept
{
    return IsColumn(position) ? GetGoods(static_cast<Column>(position)) : nullptr;
}

std::uint32_t CEquipmentContainer::GetGoodsAmount() const noexcept
{
    return static_cast<std::uint32_t>(m_Equipment.size());
}

std::uint32_t CEquipmentContainer::GetContentsWeight() const noexcept
{
    std::uint32_t weight{};
    for (const auto& [column, goods] : m_Equipment) {
        (void)column;
        weight += goods->GetWeight(m_Factory.Query(goods->GetBasePropertiesIndex()));
    }
    return weight;
}

bool CEquipmentContainer::Serialize(std::vector<std::uint8_t>& output) const
{
    if (m_Equipment.size() > std::numeric_limits<std::uint32_t>::max()) {
        return false;
    }
    AppendU32(output, static_cast<std::uint32_t>(m_Equipment.size()));
    for (const auto& [column, goods] : m_Equipment) {
        AppendU32(output, static_cast<std::uint32_t>(column));
        if (!goods->AddToByteArray(output, true)) {
            return false;
        }
    }
    return true;
}

bool CEquipmentContainer::Unserialize(std::span<const std::uint8_t> input,
                                      std::size_t& offset)
{
    Clear();
    std::uint32_t count{};
    if (!ReadU32(input, offset, count)) {
        return false;
    }
    for (std::uint32_t index = 0; index < count; ++index) {
        std::uint32_t position{};
        if (!ReadU32(input, offset, position)) {
            return false;
        }
        auto goods = m_Factory.Unserialize(input, offset);
        if (!goods || !AddFromDB(position, goods)) {
            return false;
        }
    }
    return true;
}

void CEquipmentContainer::Traverse(CContainerListener& listener)
{
    for (auto& [column, goods] : m_Equipment) {
        (void)column;
        listener.OnTraversingContainer(*this, *goods);
    }
}

void CEquipmentContainer::AI()
{
    for (auto& [column, goods] : m_Equipment) {
        (void)column;
        goods->AI();
    }
}

bool CEquipmentContainer::Accepts(Column column,
                                  CGoodsBaseProperties::EquipPlace place) noexcept
{
    if (place == CGoodsBaseProperties::EquipPlace::Ornaments) {
        return column == Column::OrnamentsOne || column == Column::OrnamentsTwo;
    }
    const auto expected = static_cast<std::uint32_t>(place);
    const auto actual = static_cast<std::uint32_t>(column);
    if (place <= CGoodsBaseProperties::EquipPlace::Boot) {
        return expected > 0 && actual == expected - 1;
    }
    if (place == CGoodsBaseProperties::EquipPlace::Jewelry) {
        return column == Column::Jewelry;
    }
    if (place == CGoodsBaseProperties::EquipPlace::Medal ||
        place == CGoodsBaseProperties::EquipPlace::Posterior) {
        return actual == expected + 1;
    }
    if (place >= CGoodsBaseProperties::EquipPlace::Headgear) {
        return actual == expected;
    }
    return false;
}

bool CEquipmentContainer::IsColumn(std::uint32_t value) noexcept
{
    return value <= static_cast<std::uint32_t>(Column::LingBao);
}
