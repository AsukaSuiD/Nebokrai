#pragma once

#include "ccontainer.h"
#include "../goods/cgoodsfactory.h"

#include <cstddef>
#include <cstdint>
#include <map>
#include <memory>
#include <span>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/container/cgoodscontainer.cpp/.h.
 * unique_ptr и std::map заменяют raw ownership и MSVC hash-map, не меняя GUID
 * identity. Порядок hash buckets не являлся wire-контрактом: каждая запись
 * несёт GUID, а callbacks стандартных контейнеров были no-op.
 */
class CGoodsContainer : public CContainer
{
public:
    using GoodsPtr = std::unique_ptr<CGoods>;

    explicit CGoodsContainer(const CGoodsFactory& factory) noexcept : m_Factory(factory) {}
    ~CGoodsContainer() override = default;

    virtual void Clear(void* context = nullptr);
    virtual void Release(void* context = nullptr);
    virtual bool Add(GoodsPtr& goods, void* context = nullptr);
    virtual bool AddFromDB(GoodsPtr& goods, std::uint32_t position = 0, void* context = nullptr);
    virtual GoodsPtr Remove(const CGUID& guid, void* context = nullptr);
    virtual GoodsPtr RemoveAt(std::uint32_t position,
                              std::uint32_t amount,
                              void* context = nullptr);

    virtual bool Serialize(std::vector<std::uint8_t>& output, bool includeChild = true) const;
    virtual bool Unserialize(std::span<const std::uint8_t> input, std::size_t& offset);
    void Traverse(CContainerListener& listener) override;
    virtual void AI();

    [[nodiscard]] virtual bool IsFull() const noexcept { return false; }
    [[nodiscard]] virtual std::uint32_t GetGoodsAmount() const noexcept;
    [[nodiscard]] virtual CGoods* GetGoods(std::uint32_t position) const noexcept;
    [[nodiscard]] CGoods* Find(const CGUID& guid) const noexcept;
    [[nodiscard]] bool QueryGoodsPosition(const CGUID& guid,
                                          std::uint32_t& position) const noexcept;
    [[nodiscard]] bool IsGoodsExisted(std::uint32_t basePropertiesIndex) const noexcept;
    [[nodiscard]] CGoods* GetFirstGoods(std::uint32_t basePropertiesIndex) const noexcept;
    void GetGoods(std::uint32_t basePropertiesIndex, std::vector<CGoods*>& output) const;

    void SetOwner(std::int32_t type, std::int32_t id) noexcept;
    [[nodiscard]] std::int32_t GetOwnerType() const noexcept { return m_OwnerType; }
    [[nodiscard]] std::int32_t GetOwnerID() const noexcept { return m_OwnerId; }

protected:
    [[nodiscard]] const CGoodsFactory& Factory() const noexcept { return m_Factory; }
    [[nodiscard]] const std::map<CGUID, GoodsPtr, guid_compare>& Goods() const noexcept { return m_Goods; }
    [[nodiscard]] std::map<CGUID, GoodsPtr, guid_compare>& Goods() noexcept { return m_Goods; }

private:
    const CGoodsFactory& m_Factory;
    std::map<CGUID, GoodsPtr, guid_compare> m_Goods;
    std::int32_t m_OwnerType{};
    std::int32_t m_OwnerId{};
};
