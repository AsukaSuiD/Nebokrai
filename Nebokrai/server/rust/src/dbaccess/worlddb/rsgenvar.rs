//! DB-владелец `CRsGenVar` исторического WorldServer из `rsgenvar.cpp`.
//!
//! Статус `Save` RVA `0x000FFE10` — `IMPLEMENTED`; constructor, destructor и
//! `Load` ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgenvar.cpp`.
//!
//! Exact EXE `0x004FFE10..0x005002C0` подтверждает цикл по всем переменным.
//! Пустое имя пропускается. Для остальных строк выполняется буквальный
//! `SELECT * FROM CSL_GENVAR WHERE VarName = '%s'`: EOF ведёт к `INSERT` трёх
//! `VarName/SValue/CValue`, существующая строка — к `UPDATE` только `CValue`.
//! Нормальный эпилог `0x005002AF` возвращает `true`, null/create/query/insert
//! failure и catch `0x005001AF` — `false` с `Save CSL_GENVAR ERROR`.
//!
//! Существенная странность сохранена: результат `ExecuteCn` для INSERT
//! проверялся, а для UPDATE по `0x00500177` игнорировался, после чего цикл
//! продолжался и итог мог быть `true`. Typed notice фиксирует эту ошибку, но
//! не меняет результат. SQL остаётся буквальным batch без escaping, поэтому
//! апостроф в runtime-значении вызывает тот же SQL-синтаксис/эффект, а не
//! незаметно исправляется параметризацией.
//!
//! Старый `_sprintf` писал batch в `char[1024]`. Вывод длиной до 1023 байт
//! воспроизводится byte-exact; для большего вывода исходный результат является
//! неизвестным UB. Безопасный Rust не назначает ему fail-closed поведение:
//! возвращает локальный `BLOCKED_MISSING_FACT` с одним вопросом о достижимости
//! и реакции оригинала, не выполняя отличающийся SQL. ANSI-байты после сборки
//! декодируются Windows-1251, как уже доказано для русской поставки, и
//! отправляются закреплённым `tiberius`; `Vec`, stream consumption и Rust Drop
//! заменяют `std::string`, ADO recordset/COM и compiler cleanup.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;

use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::worldserver::appworld::script::variablelist::VariableListSaveSource;

const LEGACY_SQL_BUFFER_CAPACITY: usize = 1024;
const SELECT_PREFIX: &[u8] = b"SELECT * FROM CSL_GENVAR WHERE VarName = '";
const SELECT_SUFFIX: &[u8] = b"'";
const INSERT_PREFIX: &[u8] = b"INSERT INTO CSL_GENVAR(VarName, SValue,CValue) VALUES('";
const INSERT_MIDDLE_INITIAL: &[u8] = b"','";
const INSERT_MIDDLE_CURRENT: &[u8] = b"','";
const INSERT_SUFFIX: &[u8] = b"')";
const UPDATE_PREFIX: &[u8] = b"UPDATE CSL_GENVAR SET CValue='";
const UPDATE_MIDDLE: &[u8] = b"' WHERE VarName = '";
const UPDATE_SUFFIX: &[u8] = b"'";

/// Вид исходного SQL, для которого не доказано поведение buffer overflow.
#[derive(Clone, Copy, Debug)]
pub(crate) enum GenVarStatement {
    Select,
    Insert,
    Update,
}

/// Локальная граница единственного ещё отсутствующего факта `CRsGenVar::Save`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GenVarSqlBufferBlock {
    pub(crate) variable_index: usize,
    pub(crate) statement: GenVarStatement,
    /// Число байт batch вместе с завершающим NUL.
    pub(crate) required_bytes: usize,
}

/// Доказанный bool-результат либо изолированная неизвестная UB-граница.
#[derive(Debug)]
pub(crate) enum GenVarSaveOutcome {
    Saved,
    Failed,
    BlockedMissingFact(GenVarSqlBufferBlock),
}

/// Этап, на котором исходный ADO-вызов отказал.
#[derive(Clone, Copy, Debug)]
pub(crate) enum GenVarDatabaseOperation {
    Select,
    Insert,
    Update,
}

/// Структурированная замена исходных `PrintErr`/SQL-file ветвей.
#[derive(Debug)]
pub(crate) enum RsGenVarNotice {
    SaveFailed {
        variable_index: usize,
        operation: GenVarDatabaseOperation,
        error: RsGenVarDatabaseError,
    },
    /// `ExecuteCn` печатал ошибку, но caller сознательно игнорировал `false`.
    UpdateFailedIgnored {
        variable_index: usize,
        error: RsGenVarDatabaseError,
    },
}

/// Ошибка достигнутой ADO/TDS-границы без runtime SQL и variable values.
#[derive(Debug)]
pub(crate) struct RsGenVarDatabaseError(tiberius::error::Error);

impl fmt::Display for RsGenVarDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "ошибка TDS World general-variable DB: {}",
            self.0
        )
    }
}

impl Error for RsGenVarDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsGenVarDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

/// Узкая объектная граница достигнутого `CRsGenVar::Save`.
pub(crate) trait RsGenVarOwner {
    /// Сохраняет список внутри уже начатой caller-транзакции.
    async fn save<S: VariableListSaveSource>(
        &mut self,
        variables: &S,
        active_transaction: &mut WorldTdsClient,
    ) -> GenVarSaveOutcome;

    /// Забирает следующий исходный log-эквивалент.
    fn pop_notice(&mut self) -> Option<RsGenVarNotice>;
}

/// Linux/TDS-замена достигнутой части исходного `CRsGenVar`.
#[derive(Default)]
pub(crate) struct TiberiusRsGenVar {
    notices: VecDeque<RsGenVarNotice>,
}

impl RsGenVarOwner for TiberiusRsGenVar {
    async fn save<S: VariableListSaveSource>(
        &mut self,
        variables: &S,
        active_transaction: &mut WorldTdsClient,
    ) -> GenVarSaveOutcome {
        for variable_index in 0..variables.variable_count() {
            let row = variables.save_row(variable_index);
            let name = visible_c_string(&row.name);
            if name.is_empty() {
                continue;
            }

            let select = match build_sql(
                variable_index,
                GenVarStatement::Select,
                &[SELECT_PREFIX, name, SELECT_SUFFIX],
            ) {
                Ok(sql) => sql,
                Err(block) => return GenVarSaveOutcome::BlockedMissingFact(block),
            };
            let missing = match query_is_empty(active_transaction, select).await {
                Ok(missing) => missing,
                Err(error) => {
                    self.notices.push_back(RsGenVarNotice::SaveFailed {
                        variable_index,
                        operation: GenVarDatabaseOperation::Select,
                        error: error.into(),
                    });
                    return GenVarSaveOutcome::Failed;
                }
            };

            if missing {
                let initial_value = visible_c_string(&row.initial_value);
                let current_value = visible_c_string(&row.current_value);
                let insert = match build_sql(
                    variable_index,
                    GenVarStatement::Insert,
                    &[
                        INSERT_PREFIX,
                        name,
                        INSERT_MIDDLE_INITIAL,
                        initial_value,
                        INSERT_MIDDLE_CURRENT,
                        current_value,
                        INSERT_SUFFIX,
                    ],
                ) {
                    Ok(sql) => sql,
                    Err(block) => return GenVarSaveOutcome::BlockedMissingFact(block),
                };
                if let Err(error) = execute_batch(active_transaction, insert).await {
                    self.notices.push_back(RsGenVarNotice::SaveFailed {
                        variable_index,
                        operation: GenVarDatabaseOperation::Insert,
                        error: error.into(),
                    });
                    return GenVarSaveOutcome::Failed;
                }
            } else {
                let current_value = visible_c_string(&row.current_value);
                let update = match build_sql(
                    variable_index,
                    GenVarStatement::Update,
                    &[
                        UPDATE_PREFIX,
                        current_value,
                        UPDATE_MIDDLE,
                        name,
                        UPDATE_SUFFIX,
                    ],
                ) {
                    Ok(sql) => sql,
                    Err(block) => return GenVarSaveOutcome::BlockedMissingFact(block),
                };
                if let Err(error) = execute_batch(active_transaction, update).await {
                    self.notices.push_back(RsGenVarNotice::UpdateFailedIgnored {
                        variable_index,
                        error: error.into(),
                    });
                }
            }
        }

        GenVarSaveOutcome::Saved
    }

    fn pop_notice(&mut self) -> Option<RsGenVarNotice> {
        self.notices.pop_front()
    }
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn build_sql(
    variable_index: usize,
    statement: GenVarStatement,
    fragments: &[&[u8]],
) -> Result<String, GenVarSqlBufferBlock> {
    let output_bytes = fragments
        .iter()
        .map(|fragment| fragment.len())
        .sum::<usize>();
    let required_bytes = output_bytes.saturating_add(1);
    if required_bytes > LEGACY_SQL_BUFFER_CAPACITY {
        return Err(GenVarSqlBufferBlock {
            variable_index,
            statement,
            required_bytes,
        });
    }

    let mut sql = Vec::with_capacity(output_bytes);
    for fragment in fragments {
        sql.extend_from_slice(fragment);
    }
    let (decoded, _, _) = WINDOWS_1251.decode(&sql);
    Ok(decoded.into_owned())
}

async fn query_is_empty(
    connection: &mut WorldTdsClient,
    sql: String,
) -> Result<bool, tiberius::error::Error> {
    Ok(connection
        .simple_query(sql)
        .await?
        .into_row()
        .await?
        .is_none())
}

async fn execute_batch(
    connection: &mut WorldTdsClient,
    sql: String,
) -> Result<(), tiberius::error::Error> {
    connection.simple_query(sql).await?.into_results().await?;
    Ok(())
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgenvar.cpp

// ============================================================================
// FUNCTION: Recordset15::AddNew
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgenvar.cpp
// RVA: 0x000ECCD0
// ADDRESS: 004eccd0
// PROTOTYPE: long __thiscall AddNew(_variant_t * param_1, _variant_t * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: CRsGenVar::Load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgenvar.cpp:23
// RVA: 0x000FF5A0
// ADDRESS: 004ff5a0
// PROTOTYPE: bool __thiscall Load(CVariableList * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ffd93
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsgenvar.cpp:65
// RVA: 0x000FFD93
// ADDRESS: 004ffd93
// PROTOTYPE: undefined Catch@004ffd93()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//















// COMPONENT_VARIANT_END: WorldServer
