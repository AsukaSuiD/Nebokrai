#include "cbank.h"

/* Исходный владелец: cbank.cpp/.h. Собственное поле — только lock-флаг. */
bool CBank::Add(std::unique_ptr<CGoods>& goods, void* context)
{
    return !m_Locked && CWallet::Add(goods, context);
}

std::unique_ptr<CGoods> CBank::Remove(const CGUID& guid, void* context)
{
    return m_Locked ? std::unique_ptr<CGoods>{} : CWallet::Remove(guid, context);
}
