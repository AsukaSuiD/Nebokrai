//! DB-владелец `CRSGodsBattle` исторического WorldServer из
//! `rsgodsbattle.cpp`.
//!
//! Статусы `SaveFacitonXYD` RVA `0x000ED0F0` и caller-connection overload
//! `SaveNpcFaction` RVA `0x000ED4B0` — `IMPLEMENTED`; constructor, destructor,
//! load-владельцы и no-argument `SaveNpcFaction` ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp`.
//!
//! Владелец сначала выполнял буквальный `DELETE FROM CSL_GODSBATTLE`, затем
//! открывал updateable table-recordset, добавлял одну строку и записывал
//! `RegionID=0`, `AFactionXYD` из signed `CShape::GetPos` и `BFactionXYD` из
//! unsigned `CGodsBattleConf::GetBFactionXYD`. Все три значения передавались
//! как `VT_UI4`; поэтому Rust явно сохраняет 32-битный шаблон A-значения и
//! связывает оба XYD как неотрицательные TDS `i64`, оставляя преобразование
//! исходной MSSQL-схеме. Явный `INSERT` заменяет только ADO `AddNew/Update`.
//!
//! Существенная странность сохранена: `ExecuteCn` сам печатал ошибку и
//! возвращал `false`, но `SaveFacitonXYD` результат DELETE не читал и всё равно
//! переходил к созданию recordset. Typed notice фиксирует такую ошибку, не
//! меняя итог; только ошибка следующего INSERT возвращает `false`. Метод
//! использует caller-owned соединение внутри уже активной транзакции и сам не
//! выполняет begin/commit/rollback.
//!
//! Exact EXE имеет статус `VERIFIED_DISASSEMBLY`: вызов `ExecuteCn` в
//! `0x004ED1A5` сразу продолжается в `CreateRs`; successful `Update` ставит
//! `AL=1` в `0x004ED436`, а null connection и catch сходятся в `xor al, al` по
//! `0x004ED484`. После ответа reverse прекращён. `Option` сохраняет null
//! caller-connection, snapshot заменяет только чтение уже существующего
//! singleton, а `VecDeque`, Tiberius и Rust `Drop` — STL/ADO/COM/compiler
//! cleanup. Raw реализованного владельца, его catch и служебный эпилог удалены.
//!
//! Caller-connection `SaveNpcFaction` сначала так же игнорировал `false` от
//! буквального `DELETE FROM CSL_GODSBATTLE_NPC`, затем до проверки размера
//! `CGodsBattleConf::m_vNpcName` открывал updateable table-recordset. Поэтому
//! Tiberius отдельно проверяет доступность таблицы даже для пустого snapshot.
//! Для каждого элемента в исходном vector-order выполнялся `AddNew`, C-string
//! prefix `name` записывался в `NPC_NAME`, signed `long faction` как `VT_I4` —
//! в колонку с исторической опечаткой `Faciton`, затем вызывался `Update`.
//! `monsters` этого DB-owner-а не касался.
//!
//! Exact PDB задаёт `_FactionNpcName` размером `0x3C`: `long faction` по `+0`,
//! `std::string name` по `+4`, `std::string monsters` по `+0x20`.
//! `VERIFIED_DISASSEMBLY` подтверждает игнорирование результата `ExecuteCn` по
//! `0x004ED541`, normal `AL=1` по `0x004ED92B` и общий `AL=0` для null/catch
//! по `0x004ED97D`; после этих ответов reverse прекращён. Windows-1251,
//! параметризованный INSERT и caller-owned ordered slice заменяют только
//! BSTR/ADO, table-recordset и MSVC vector storage. Raw этого overload-а, его
//! catch и compiler-эпилог удалены; отдельный no-argument overload не смешан.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;
use tiberius::Query;

use crate::dbaccess::worlddb::rssetup::WorldTdsClient;

const DELETE_FACTION_XYD_SQL: &str = "DELETE FROM CSL_GODSBATTLE";
const INSERT_FACTION_XYD_SQL: &str =
    "INSERT INTO CSL_GODSBATTLE (RegionID, AFactionXYD, BFactionXYD) VALUES (@P1, @P2, @P3)";
const DELETE_NPC_FACTIONS_SQL: &str = "DELETE FROM CSL_GODSBATTLE_NPC";
const OPEN_NPC_FACTIONS_SQL: &str = "SELECT TOP 0 * FROM CSL_GODSBATTLE_NPC";
const INSERT_NPC_FACTION_SQL: &str =
    "INSERT INTO CSL_GODSBATTLE_NPC (NPC_NAME, Faciton) VALUES (@P1, @P2)";

/// Два значения, которые исходный DB-владелец читал из `CGodsBattleConf`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GodsBattleFactionXydSnapshot {
    pub(crate) a_faction_xyd: i32,
    pub(crate) b_faction_xyd: u32,
}

/// Одна запись исходного ordered `m_vNpcName`.
#[derive(Clone, Debug)]
pub(crate) struct GodsBattleNpcFactionSnapshot {
    pub(crate) faction: i32,
    pub(crate) name: Vec<u8>,
}

/// Достигнутый save-владелец, создавший log-эквивалент.
#[derive(Clone, Copy, Debug)]
pub(crate) enum GodsBattleSaveOperation {
    FactionXyd,
    NpcFaction,
}

/// Структурированная замена достигнутых log-ветвей `CRSGodsBattle`.
#[derive(Debug)]
pub(crate) enum RsGodsBattleNotice {
    MissingConnection {
        operation: GodsBattleSaveOperation,
    },
    DeleteFailedIgnored {
        operation: GodsBattleSaveOperation,
        error: RsGodsBattleDatabaseError,
    },
    SaveFailed {
        operation: GodsBattleSaveOperation,
        row_index: Option<usize>,
        error: RsGodsBattleDatabaseError,
    },
}

/// Ошибка достигнутой ADO/TDS-границы без runtime SQL и значений строк.
#[derive(Debug)]
pub(crate) struct RsGodsBattleDatabaseError(tiberius::error::Error);

impl fmt::Display for RsGodsBattleDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World GodsBattle DB: {}", self.0)
    }
}

impl Error for RsGodsBattleDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsGodsBattleDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

/// Узкая объектная граница достигнутых save-функций `CRSGodsBattle`.
pub(crate) trait RsGodsBattleOwner {
    /// Заменяет единственную faction-XYD строку внутри caller-транзакции.
    async fn save_faction_xyd(
        &mut self,
        snapshot: GodsBattleFactionXydSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Заменяет все NPC-faction строки в исходном vector-order.
    async fn save_npc_faction(
        &mut self,
        snapshot: &[GodsBattleNpcFactionSnapshot],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Забирает следующий исходный log-эквивалент.
    fn pop_notice(&mut self) -> Option<RsGodsBattleNotice>;
}

/// Linux/TDS-замена достигнутой части исходного `CRSGodsBattle`.
#[derive(Default)]
pub(crate) struct TiberiusRsGodsBattle {
    notices: VecDeque<RsGodsBattleNotice>,
}

impl TiberiusRsGodsBattle {
    /// Ставит границу notice-очереди перед отдельным synchronous owner-call.
    pub(crate) fn notice_checkpoint(&self) -> usize {
        self.notices.len()
    }

    /// Забирает только notices, созданные после checkpoint, сохраняя прежние.
    pub(crate) fn drain_notices_after(
        &mut self,
        checkpoint: usize,
    ) -> Vec<RsGodsBattleNotice> {
        if checkpoint >= self.notices.len() {
            return Vec::new();
        }
        self.notices.drain(checkpoint..).collect()
    }
}

impl RsGodsBattleOwner for TiberiusRsGodsBattle {
    async fn save_faction_xyd(
        &mut self,
        snapshot: GodsBattleFactionXydSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(active_transaction) = active_transaction else {
            self.notices
                .push_back(RsGodsBattleNotice::MissingConnection {
                    operation: GodsBattleSaveOperation::FactionXyd,
                });
            return false;
        };

        if let Err(error) = execute_batch(active_transaction, DELETE_FACTION_XYD_SQL).await {
            self.notices
                .push_back(RsGodsBattleNotice::DeleteFailedIgnored {
                    operation: GodsBattleSaveOperation::FactionXyd,
                    error: error.into(),
                });
        }

        let mut insert = Query::new(INSERT_FACTION_XYD_SQL);
        insert.bind(0_i64);
        insert.bind(i64::from(snapshot.a_faction_xyd as u32));
        insert.bind(i64::from(snapshot.b_faction_xyd));
        match insert.execute(active_transaction).await {
            Ok(_) => true,
            Err(error) => {
                self.notices.push_back(RsGodsBattleNotice::SaveFailed {
                    operation: GodsBattleSaveOperation::FactionXyd,
                    row_index: None,
                    error: error.into(),
                });
                false
            }
        }
    }

    async fn save_npc_faction(
        &mut self,
        snapshot: &[GodsBattleNpcFactionSnapshot],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(active_transaction) = active_transaction else {
            self.notices
                .push_back(RsGodsBattleNotice::MissingConnection {
                    operation: GodsBattleSaveOperation::NpcFaction,
                });
            return false;
        };

        if let Err(error) = execute_batch(active_transaction, DELETE_NPC_FACTIONS_SQL).await {
            self.notices
                .push_back(RsGodsBattleNotice::DeleteFailedIgnored {
                    operation: GodsBattleSaveOperation::NpcFaction,
                    error: error.into(),
                });
        }

        if let Err(error) = execute_batch(active_transaction, OPEN_NPC_FACTIONS_SQL).await {
            self.notices.push_back(RsGodsBattleNotice::SaveFailed {
                operation: GodsBattleSaveOperation::NpcFaction,
                row_index: None,
                error: error.into(),
            });
            return false;
        }

        for (row_index, row) in snapshot.iter().enumerate() {
            let (name, _, _) = WINDOWS_1251.decode(visible_c_string(&row.name));
            let mut insert = Query::new(INSERT_NPC_FACTION_SQL);
            insert.bind(name.into_owned());
            insert.bind(row.faction);
            if let Err(error) = insert.execute(&mut *active_transaction).await {
                self.notices.push_back(RsGodsBattleNotice::SaveFailed {
                    operation: GodsBattleSaveOperation::NpcFaction,
                    row_index: Some(row_index),
                    error: error.into(),
                });
                return false;
            }
        }

        true
    }

    fn pop_notice(&mut self) -> Option<RsGodsBattleNotice> {
        self.notices.pop_front()
    }
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp



// ============================================================================
// FUNCTION: CRSGodsBattle::LoadFactionXYD
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:16
// RVA: 0x000ECD60
// ADDRESS: 004ecd60
// PROTOTYPE: bool __thiscall LoadFactionXYD(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ed06c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:40
// RVA: 0x000ED06C
// ADDRESS: 004ed06c
// PROTOTYPE: undefined Catch@004ed06c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004ed0cb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:50
// RVA: 0x000ED0CB
// ADDRESS: 004ed0cb
// PROTOTYPE: undefined FUN_004ed0cb()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRSGodsBattle::SaveNpcFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:229
// RVA: 0x000ED9A0
// ADDRESS: 004ed9a0
// PROTOTYPE: bool __thiscall SaveNpcFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ede82
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:266
// RVA: 0x000EDE82
// ADDRESS: 004ede82
// PROTOTYPE: undefined Catch@004ede82()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004edee1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:273
// RVA: 0x000EDEE1
// ADDRESS: 004edee1
// PROTOTYPE: undefined FUN_004edee1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRSGodsBattle::GetNpcFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:147
// RVA: 0x000EDF00
// ADDRESS: 004edf00
// PROTOTYPE: bool __thiscall GetNpcFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ee367
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:175
// RVA: 0x000EE367
// ADDRESS: 004ee367
// PROTOTYPE: undefined Catch@004ee367()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004ee3c7
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:185
// RVA: 0x000EE3C7
// ADDRESS: 004ee3c7
// PROTOTYPE: undefined FUN_004ee3c7()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRSGodsBattle::GetTopTenSZLPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:93
// RVA: 0x000EE3E0
// ADDRESS: 004ee3e0
// PROTOTYPE: bool __thiscall GetTopTenSZLPlayer(long param_1, vector<unsigned_char,std::allocator<unsigned_char>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004eea7e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:134
// RVA: 0x000EEA7E
// ADDRESS: 004eea7e
// PROTOTYPE: undefined Catch@004eea7e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004eeadd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp:144
// RVA: 0x000EEADD
// ADDRESS: 004eeadd
// PROTOTYPE: undefined FUN_004eeadd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Command15::Execute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp
// RVA: 0x00116420
// ADDRESS: 00516420
// PROTOTYPE: _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> __thiscall Execute(tagVARIANT * param_1, tagVARIANT * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Command15::GetParameters
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp
// RVA: 0x00116480
// ADDRESS: 00516480
// PROTOTYPE: _com_ptr_t<_com_IIID<Parameters,&struct___s_GUID_const__GUID_0000050d_0000_0010_8000_00aa006d2ea4>_> __thiscall GetParameters(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Command15::PutCommandText
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgodsbattle.cpp
// RVA: 0x00116920
// ADDRESS: 00516920
// PROTOTYPE: void __thiscall PutCommandText(_bstr_t param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
















// COMPONENT_VARIANT_END: WorldServer
