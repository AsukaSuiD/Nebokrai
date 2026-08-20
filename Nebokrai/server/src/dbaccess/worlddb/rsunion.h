#pragma once
#include "rssetup.h"
#include "../../world/appworld/organizingsystem/union.h"

/* Исходный владелец: dbaccess/worlddb/rsunion.cpp/.h. Save/Delete и member
 * replace подтверждены Rust; null union больше не разыменовывается. */
class CRsUnion{public:static WorldDbResult Load(IWorldDbExecutor&);static WorldDbResult LoadMembers(IWorldDbExecutor&);static bool Save(IWorldDbExecutor&,const UnionSaveSnapshot&);static WorldDbResult Delete(IWorldDbExecutor&,std::int32_t);};
