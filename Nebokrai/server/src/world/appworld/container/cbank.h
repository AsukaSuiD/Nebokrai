#pragma once

#include "cwallet.h"

class CBank final : public CWallet
{
public:
    explicit CBank(const CGoodsFactory& factory)
        : CWallet(factory, factory.GoldCoinIndex()) {}

    bool Add(std::unique_ptr<CGoods>& goods, void* context = nullptr) override;
    std::unique_ptr<CGoods> Remove(const CGUID& guid, void* context = nullptr) override;
    void SetLocked(bool value) noexcept { m_Locked = value; }
    [[nodiscard]] bool IsLocked() const noexcept { return m_Locked; }

private:
    bool m_Locked{};
};
