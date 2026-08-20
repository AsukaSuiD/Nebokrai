//! DB-владелец `CRsEnemyFactions` исторического WorldServer из
//! `rsenemyfactions.cpp`.
//!
//! Статус `SaveAllEnemyFactions` RVA `0x000F7C50` — `IMPLEMENTED` с локальной
//! `BLOCKED_MISSING_FACT` границей для null-элемента; constructor, destructor,
//! `LoadAllEnemyFactions` и прочий корпус ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsenemyfactions.cpp`.
//!
//! Владелец выполнял буквальный `DELETE FROM CSL_FactionWar`, затем обходил
//! переданную по значению копию `std::list<tagEnemyFaction*>` в list-order и
//! для каждой записи выполнял
//! `INSERT INTO CSL_FactionWar VALUES(%d,%d,%d)`. Первая ошибка DELETE либо
//! INSERT немедленно возвращала `false`; после всех строк возвращался `true`.
//! Метод использовал caller-owned connection внутри уже начатой транзакции и
//! сам не выполнял begin/commit/rollback.
//!
//! Exact PDB задаёт `tagEnemyFaction` размером `0x0C`: signed `long`
//! `lFactionID1` по `+0`, signed `long lFactionID2` по `+4` и unsigned `long`
//! `dwDisandTime` по `+8`; локальный SQL-буфер имел размер `0x1F4`. Exact EXE
//! имеет статус `VERIFIED_DISASSEMBLY`: call-site `0x004F7D1B..0x004F7D35`
//! передаёт третьим `%d` именно поле `+8`, поэтому Rust сохраняет исходную
//! signed decimal-интерпретацию через `u32 as i32`. Выходы `0x004F7CF9`,
//! `0x004F7DAD` и catch-путь `0x004F7E11` ставят `AL=0`, а полный успех
//! `0x004F7E7B` — `AL=1`. После этих ответов reverse прекращён.
//!
//! `Option` сохраняет nullable connection и nullable элементы pointer-list.
//! Для null-элемента EXE без проверки разыменовывал `node->_Myval` после уже
//! успешного DELETE и возможных предыдущих INSERT. Достижимость и дальнейший
//! эффект такого UB не доказаны, поэтому безопасная граница возвращает только
//! индекс и не назначает исходнику `false`, skip либо commit. Ordered slice,
//! Tiberius и Rust `Drop` заменяют копию `std::list`, ADO/COM и compiler
//! cleanup; неиспользуемый null recordset и raw реализованного владельца,
//! catch и служебный эпилог удалены.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use tiberius::Query;

use crate::dbaccess::worlddb::rssetup::WorldTdsClient;

const DELETE_ENEMY_FACTIONS_SQL: &str = "DELETE FROM CSL_FactionWar";
const INSERT_ENEMY_FACTION_SQL: &str = "INSERT INTO CSL_FactionWar VALUES(@P1,@P2,@P3)";

/// Три точных 32-битных поля одной caller-owned save-копии.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EnemyFactionSaveSnapshot {
    pub(crate) faction_id_1: i32,
    pub(crate) faction_id_2: i32,
    pub(crate) disband_time: u32,
}

/// Локальная неизвестность исходного null-разыменования внутри ordered списка.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EnemyFactionNullEntryBlock {
    pub(crate) row_index: usize,
}

/// Наблюдаемый bool-результат владельца либо локальный missing-fact.
#[derive(Debug)]
pub(crate) enum EnemyFactionsSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(EnemyFactionNullEntryBlock),
}

/// Структурированная замена достигнутых `PrintErr`-ветвей владельца.
#[derive(Debug)]
pub(crate) enum RsEnemyFactionsNotice {
    MissingConnection,
    DeleteFailed(RsEnemyFactionsDatabaseError),
    InsertFailed {
        row_index: usize,
        error: RsEnemyFactionsDatabaseError,
    },
}

/// Ошибка достигнутой ADO/TDS-границы без runtime SQL и значений строк.
#[derive(Debug)]
pub(crate) struct RsEnemyFactionsDatabaseError(tiberius::error::Error);

impl fmt::Display for RsEnemyFactionsDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World enemy factions DB: {}", self.0)
    }
}

impl Error for RsEnemyFactionsDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsEnemyFactionsDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

/// Узкая объектная граница достигнутого `CRsEnemyFactions` save-владельца.
pub(crate) trait RsEnemyFactionsOwner {
    /// Полностью заменяет строки `CSL_FactionWar` внутри caller-транзакции.
    async fn save_all_enemy_factions(
        &mut self,
        snapshot: &[Option<EnemyFactionSaveSnapshot>],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> EnemyFactionsSaveOutcome;

    /// Забирает следующий исходный log-эквивалент.
    fn pop_notice(&mut self) -> Option<RsEnemyFactionsNotice>;
}

/// Linux/TDS-замена достигнутой части исходного `CRsEnemyFactions`.
#[derive(Default)]
pub(crate) struct TiberiusRsEnemyFactions {
    notices: VecDeque<RsEnemyFactionsNotice>,
}

impl RsEnemyFactionsOwner for TiberiusRsEnemyFactions {
    async fn save_all_enemy_factions(
        &mut self,
        snapshot: &[Option<EnemyFactionSaveSnapshot>],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> EnemyFactionsSaveOutcome {
        let Some(active_transaction) = active_transaction else {
            self.notices
                .push_back(RsEnemyFactionsNotice::MissingConnection);
            return EnemyFactionsSaveOutcome::ReturnedFalse;
        };

        if let Err(error) = execute_batch(active_transaction, DELETE_ENEMY_FACTIONS_SQL).await {
            self.notices
                .push_back(RsEnemyFactionsNotice::DeleteFailed(error.into()));
            return EnemyFactionsSaveOutcome::ReturnedFalse;
        }

        for (row_index, row) in snapshot.iter().enumerate() {
            let Some(row) = row else {
                // BLOCKED_MISSING_FACT: что наблюдалось после разыменования null
                // в WorldServer RVA 0x000F7D1B? Минимум: `mov eax,[edi+8]`;
                // `mov ecx,[eax+8]`. DELETE и предыдущие INSERT уже выполнены.
                return EnemyFactionsSaveOutcome::BlockedMissingFact(EnemyFactionNullEntryBlock {
                    row_index,
                });
            };

            let mut insert = Query::new(INSERT_ENEMY_FACTION_SQL);
            insert.bind(row.faction_id_1);
            insert.bind(row.faction_id_2);
            insert.bind(row.disband_time as i32);
            if let Err(error) = insert.execute(&mut *active_transaction).await {
                self.notices.push_back(RsEnemyFactionsNotice::InsertFailed {
                    row_index,
                    error: error.into(),
                });
                return EnemyFactionsSaveOutcome::ReturnedFalse;
            }
        }

        EnemyFactionsSaveOutcome::ReturnedTrue
    }

    fn pop_notice(&mut self) -> Option<RsEnemyFactionsNotice> {
        self.notices.pop_front()
    }
}

async fn execute_batch(
    connection: &mut WorldTdsClient,
    sql: &'static str,
) -> Result<(), tiberius::error::Error> {
    connection.simple_query(sql).await?.into_results().await?;
    Ok(())
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsenemyfactions.cpp



// ============================================================================
// FUNCTION: CRsEnemyFactions::LoadAllEnemyFactions
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsenemyfactions.cpp:21
// RVA: 0x000F7800
// ADDRESS: 004f7800
// PROTOTYPE: bool __thiscall LoadAllEnemyFactions(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f7bd4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsenemyfactions.cpp:46
// RVA: 0x000F7BD4
// ADDRESS: 004f7bd4
// PROTOTYPE: undefined Catch@004f7bd4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004f7c33
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsenemyfactions.cpp:54
// RVA: 0x000F7C33
// ADDRESS: 004f7c33
// PROTOTYPE: undefined FUN_004f7c33()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



































// COMPONENT_VARIANT_END: WorldServer
