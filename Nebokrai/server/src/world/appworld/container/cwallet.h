#pragma once

#include "ccontainer.h"
#include "../goods/cgoodsfactory.h"

#include <cstddef>
#include <cstdint>
#include <memory>
#include <span>
#include <vector>

/*
 * Исходный владелец: cwallet.cpp/.h. Wallet содержит ровно один nullable
 * currency-goods slot; marker 0/1 и затем полный CGoods — подтверждённый wire
 * формат. unique_ptr заменяет только старый deleting lifecycle.
 */
class CWallet : public CContainer
{
public:
    CWallet(const CGoodsFactory& factory, std::uint32_t currencyIndex) noexcept;
    ~CWallet() override = default;

    virtual bool Add(std::unique_ptr<CGoods>& goods, void* context = nullptr);
    virtual bool AddFromDB(std::unique_ptr<CGoods>& goods,
                           std::uint32_t position = 0,
                           void* context = nullptr);
    virtual std::unique_ptr<CGoods> Remove(const CGUID& guid, void* context = nullptr);
    virtual void Clear(void* context = nullptr);
    virtual void Release(void* context = nullptr);
    [[nodiscard]] virtual bool IsFull() const noexcept;
    [[nodiscard]] CGoods* GetGoods(std::uint32_t position = 0) const noexcept;
    [[nodiscard]] std::uint32_t GetGoodsAmount() const noexcept;
    [[nodiscard]] std::uint32_t GetCurrencyAmount() const noexcept;
    [[nodiscard]] bool IsGoodsExisted(std::uint32_t basePropertiesIndex) const noexcept;
    bool Serialize(std::vector<std::uint8_t>& output) const;
    bool Unserialize(std::span<const std::uint8_t> input, std::size_t& offset);
    void Traverse(CContainerListener& listener) override;

protected:
    [[nodiscard]] const CGoodsFactory& Factory() const noexcept { return m_Factory; }
    [[nodiscard]] std::uint32_t CurrencyIndex() const noexcept { return m_CurrencyIndex; }

private:
    const CGoodsFactory& m_Factory;
    std::uint32_t m_CurrencyIndex{};
    std::unique_ptr<CGoods> m_Currency;
};
