#pragma once
#include "rssetup.h"
#include <cstdint>
#include <vector>

/* Исходный владелец: dbaccess/worlddb/rsenemyfactions.cpp/.h. Replace сохраняет
 * исходную транзакционную семантику DELETE всех отношений, затем ordered INSERT. */
struct EnemyFactionDbRow{std::int32_t first{},second{};std::uint32_t leaveTime{};};
class CRsEnemyFactions{public:static WorldDbResult Load(IWorldDbExecutor&);static bool Replace(IWorldDbExecutor&,const std::vector<EnemyFactionDbRow>&);};
