#pragma once
#include "rssetup.h"
#include "../../world/appworld/organizingsystem/faction.h"

/* Исходный владелец: dbaccess/worlddb/rsfaction.cpp/.h. Save принимает owned
 * faction snapshot и параметризует base row; member/rights rows остаются
 * отдельной ordered фазой, чтобы ошибка не скрывалась частичным успехом. */
class CRsFaction{public:static WorldDbResult Load(IWorldDbExecutor&);static WorldDbResult LoadMembers(IWorldDbExecutor&);static bool Save(IWorldDbExecutor&,const FactionSaveSnapshot&);static WorldDbResult Delete(IWorldDbExecutor&,std::int32_t);};
