#include "cwallet.h"

#include <algorithm>

CWallet::CWallet(const CGoodsFactory& factory, std::uint32_t currencyIndex) noexcept
    : m_Factory(factory), m_CurrencyIndex(currencyIndex)
{
}

bool CWallet::Add(std::unique_ptr<CGoods>& goods, void* context)
{
    if (!goods || goods->GetBasePropertiesIndex() != m_CurrencyIndex) {
        return false;
    }
    if (!m_Currency) {
        m_Currency = std::move(goods);
        NotifyAdded(*m_Currency, context);
        return true;
    }
    const auto* properties = m_Factory.Query(m_CurrencyIndex);
    const std::uint32_t limit = m_Currency->GetMaxStackNumber(properties);
    if (goods->GetAmount() > limit - std::min(limit, m_Currency->GetAmount())) {
        return false;
    }
    m_Currency->SetAmount(m_Currency->GetAmount() + goods->GetAmount());
    goods.reset();
    NotifyAdded(*m_Currency, context);
    return true;
}

bool CWallet::AddFromDB(std::unique_ptr<CGoods>& goods,
                        std::uint32_t position,
                        void* context)
{
    return position == 0 && Add(goods, context);
}

std::unique_ptr<CGoods> CWallet::Remove(const CGUID& guid, void* context)
{
    if (!m_Currency || m_Currency->GetGUID() != guid) {
        return {};
    }
    NotifyRemoved(*m_Currency, context);
    return std::move(m_Currency);
}

void CWallet::Clear(void* context)
{
    if (m_Currency) {
        NotifyRemoved(*m_Currency, context);
        m_Currency.reset();
    }
}

void CWallet::Release(void* context)
{
    Clear(context);
}

bool CWallet::IsFull() const noexcept
{
    return m_Currency &&
           m_Currency->GetAmount() >= m_Currency->GetMaxStackNumber(m_Factory.Query(m_CurrencyIndex));
}

CGoods* CWallet::GetGoods(std::uint32_t position) const noexcept
{
    return position == 0 ? m_Currency.get() : nullptr;
}

std::uint32_t CWallet::GetGoodsAmount() const noexcept
{
    return m_Currency ? 1U : 0U;
}

std::uint32_t CWallet::GetCurrencyAmount() const noexcept
{
    return m_Currency ? m_Currency->GetAmount() : 0U;
}

bool CWallet::IsGoodsExisted(std::uint32_t basePropertiesIndex) const noexcept
{
    return m_Currency && basePropertiesIndex == m_CurrencyIndex;
}

bool CWallet::Serialize(std::vector<std::uint8_t>& output) const
{
    output.push_back(m_Currency ? 1 : 0);
    return !m_Currency || m_Currency->AddToByteArray(output, true);
}

bool CWallet::Unserialize(std::span<const std::uint8_t> input, std::size_t& offset)
{
    Clear();
    if (offset >= input.size()) {
        return false;
    }
    const std::uint8_t marker = input[offset++];
    if (marker == 0) {
        return true;
    }
    m_Currency = m_Factory.Unserialize(input, offset);
    return m_Currency && m_Currency->GetBasePropertiesIndex() == m_CurrencyIndex;
}

void CWallet::Traverse(CContainerListener& listener)
{
    if (m_Currency) {
        listener.OnTraversingContainer(*this, *m_Currency);
    }
}
