#pragma once

#include "officer.h"

/*
 * Исходный владелец: WorldServer/appworld/country/minister.cpp/.h.
 * У класса нет отдельной подтверждённой семантики сверх Officer; отдельный
 * тип сохраняет PDB-контракт и не переносит compiler-generated lifecycle.
 */
class CMinister final : public COfficer {};
