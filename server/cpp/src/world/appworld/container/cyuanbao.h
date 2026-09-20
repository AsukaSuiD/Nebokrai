#pragma once

#include "cwallet.h"

class CYuanBao final : public CWallet
{
public:
    explicit CYuanBao(const CGoodsFactory& factory)
        : CWallet(factory, factory.YuanBaoIndex()) {}
};
