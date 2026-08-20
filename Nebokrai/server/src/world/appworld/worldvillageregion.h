#pragma once

#include "worldwarregion.h"

class CWorldVillageRegion final : public CWorldWarRegion
{
public:
    CWorldVillageRegion() { SetSymbols(1, 1, 1); }
};
