#pragma once

#include "../container/ccontainer.h"

// Совместимый owner исходного пустого container-listener-а. Конкретные
// DB/player listeners наследуют CContainerListener непосредственно.
class CNoopContainerListener final : public CContainerListener
{
};
