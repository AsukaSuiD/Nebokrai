#include "rsenemyfactions.h"
WorldDbResult CRsEnemyFactions::Load(IWorldDbExecutor&db){return db.Execute({"SELECT * FROM CSL_FactionWar",{}});}
bool CRsEnemyFactions::Replace(IWorldDbExecutor&db,const std::vector<EnemyFactionDbRow>&rows){if(!db.Execute({"DELETE FROM CSL_FactionWar",{}}).success)return false;for(const auto&r:rows)if(!db.Execute({"INSERT INTO CSL_FactionWar VALUES(@P1,@P2,@P3)",{static_cast<std::int64_t>(r.first),static_cast<std::int64_t>(r.second),static_cast<std::uint64_t>(r.leaveTime)}}).success)return false;return true;}
