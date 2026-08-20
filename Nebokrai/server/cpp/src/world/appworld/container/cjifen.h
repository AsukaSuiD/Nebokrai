#pragma once

#include "cwallet.h"

class CJiFen final : public CWallet
{
public:
    explicit CJiFen(const CGoodsFactory& factory)
        : CWallet(factory, factory.JiFenIndex()) {}
};
