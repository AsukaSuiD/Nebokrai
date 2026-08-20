#pragma once

#include "cvolumelimitgoodscontainer.h"

class CBattleFairyContainer final : public CVolumeLimitGoodsContainer
{
public:
    explicit CBattleFairyContainer(const CGoodsFactory& factory)
        : CVolumeLimitGoodsContainer(factory) {}
};
