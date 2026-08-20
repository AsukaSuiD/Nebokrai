#pragma once
#include "playerdataqueue.h"

/* Исходный владелец: dbaccess/worlddb/playerloadqueue.cpp/.h. Load queue
 * использует тот же доказанный FIFO/close контракт, но остаётся отдельным
 * типом, чтобы save и load workers нельзя было перепутать. */
class CPlayerLoadQueue final : public CPlayerDataQueue {};
