//! DB-владелец сброса LeiTing WorldServer (`CRsPlayer::ResetAllLeitingInDB`
//! и его worker `DbLetTingUpdate`), извлечённый из мирового адаптера
//! `rsplayer`. Источник контракта — точная пара `Nworldserver.exe` +
//! `WorldServer.pdb`.
//!
//! Каждый вызов строит собственное TDS-соединение, а каждый `UPDATE` остаётся
//! отдельным statement в исходном порядке без rollback, как в оригинале.

use std::collections::VecDeque;

use chrono::{Datelike, Local};
use tiberius::Query;

use crate::persistence::row::get_value;
use crate::persistence::rssetup::{WorldDatabaseConnectionError, WorldDatabaseSettings};
use nebokrai_shared::resources::{CThingSetup, LeiTingDailyThing};

pub const LEI_TING_RESET_SELECT_SQL: &str = "SELECT ID, LTUp60Cnt, basefyEnergy, baseblfyenergy, dwLT60Stamp, wRemainJLDanCnt, ListThing FROM csl_player_ability";
pub const LEI_TING_DAILY_RESET_SQL: &str = "UPDATE CSL_PLAYER_ABILITY SET ListThing=@P1,dwLT60Stamp=@P2,basefyEnergy=@P3,wRemainJLDanCnt=@P4,baseblfyenergy=@P5 WHERE ID=@P6";
pub const LEI_TING_MONTHLY_RESET_SQL: &str = "UPDATE CSL_PLAYER_ABILITY SET ListThing=@P1,dwLT60Stamp=@P2,basefyEnergy=@P3,wRemainJLDanCnt=@P4,LTUp60Cnt=@P5,baseblfyenergy=@P6 WHERE ID=@P7";

/// Исходный двух-DWORD payload `ResetAllLeitingInDB`.
///
/// В EXE worker принимает его как `void *`: первое слово — kind, второе —
/// signed `mktime` stamp. Rust не переносит heap-allocation без владельца,
/// но сохраняет порядок и ширину обоих полей в явном значении.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeiTingDatabaseResetRequest {
    pub update_kind: u32,
    pub stamp: i32,
}

#[derive(Debug)]
pub enum LeiTingDatabaseResetOutcome {
    ReturnedTrue { updated_rows: usize },
    ReturnedFalse(LeiTingDatabaseResetFailure),
}

#[derive(Debug)]
pub enum LeiTingDatabaseResetFailure {
    UnsupportedUpdateKind(u32),
    Connection(WorldDatabaseConnectionError),
    Database(tiberius::error::Error),
    MissingRequiredValue { column: &'static str },
}

fn encode_lei_ting_daily_things(things: &VecDeque<LeiTingDailyThing>) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(things.len() * 8);
    for thing in things {
        bytes.extend_from_slice(&thing.thing_id.to_le_bytes());
        bytes.extend_from_slice(&thing.count.to_le_bytes());
        bytes.extend_from_slice(&thing.max_count.to_le_bytes());
        bytes.extend_from_slice(&thing.point.to_le_bytes());
    }
    bytes
}

pub struct TiberiusLeiTingReset {
    settings: WorldDatabaseSettings,
}

impl TiberiusLeiTingReset {
    pub fn new(settings: &WorldDatabaseSettings) -> Self {
        Self { settings: settings.clone() }
    }

 /// Выполняет один фоновый проход `DbLetTingUpdate` на отдельном соединении.
 ///
 /// Оригинал строит `ListThing` до `CreateCn/OpenCn`, затем без
 /// transaction проходит updatable recordset `csl_player_ability`; успешные
 /// ранние строки сохраняются даже если последующая строка даёт ошибку.
 /// TDS не предоставляет этот ADO recordset API, поэтому `ID` добавлен
 /// только как технический ключ текущей строки, а каждый `UPDATE` остаётся
 /// отдельным statement в том же исходном порядке без `ORDER BY` и rollback.
 /// Снятие list происходит внутри worker-вызова, а не при его постановке.
    pub async fn db_lei_ting_update(
        &self,
        request: LeiTingDatabaseResetRequest,
        thing_setup: &CThingSetup,
        total_jing_li_dan_count: u16,
    ) -> LeiTingDatabaseResetOutcome {
        if request.update_kind != 1 && request.update_kind != 2 {
            return LeiTingDatabaseResetOutcome::ReturnedFalse(
                LeiTingDatabaseResetFailure::UnsupportedUpdateKind(request.update_kind),
            );
        }

        let mut daily_things = VecDeque::new();
        thing_setup.get_daily_thing_list(
            || Local::now().weekday().num_days_from_sunday() as u16,
            &mut daily_things,
        );
        let list_thing = encode_lei_ting_daily_things(&daily_things);

        let mut connection = match self.settings.connect().await {
            Ok(connection) => connection,
            Err(error) => {
                return LeiTingDatabaseResetOutcome::ReturnedFalse(
                    LeiTingDatabaseResetFailure::Connection(error),
                );
            }
        };
        let rows = match connection.simple_query(LEI_TING_RESET_SELECT_SQL).await {
            Ok(stream) => match stream.into_first_result().await {
                Ok(rows) => rows,
                Err(error) => {
                    return LeiTingDatabaseResetOutcome::ReturnedFalse(
                        LeiTingDatabaseResetFailure::Database(error),
                    );
                }
            },
            Err(error) => {
                return LeiTingDatabaseResetOutcome::ReturnedFalse(
                    LeiTingDatabaseResetFailure::Database(error),
                );
            }
        };

        let mut updated_rows = 0;
        for row in rows {
            let player_id = match get_value::<i32>(&row, "ID") {
                Ok(Some(value)) => value,
                Ok(None) => {
                    return LeiTingDatabaseResetOutcome::ReturnedFalse(
                        LeiTingDatabaseResetFailure::MissingRequiredValue { column: "ID" },
                    );
                }
                Err(error) => {
                    return LeiTingDatabaseResetOutcome::ReturnedFalse(
                        LeiTingDatabaseResetFailure::Database(error),
                    );
                }
            };
            let base_bl_fy_energy = if request.update_kind == 1 {
                match get_value::<i32>(&row, "baseblfyenergy") {
                    Ok(Some(value)) => value as u32,
                    Ok(None) => {
                        return LeiTingDatabaseResetOutcome::ReturnedFalse(
                            LeiTingDatabaseResetFailure::MissingRequiredValue {
                                column: "baseblfyenergy",
                            },
                        );
                    }
                    Err(error) => {
                        return LeiTingDatabaseResetOutcome::ReturnedFalse(
                            LeiTingDatabaseResetFailure::Database(error),
                        );
                    }
                }
            } else {
                0
            };

            let query = if request.update_kind == 1 {
                let mut query = Query::new(LEI_TING_DAILY_RESET_SQL);
                query.bind(list_thing.as_slice());
                query.bind(request.stamp);
                query.bind(0_i64);
                query.bind(i32::from(total_jing_li_dan_count));
                query.bind(i64::from(base_bl_fy_energy & !0x0f));
                query.bind(player_id);
                query
            } else {
                let mut query = Query::new(LEI_TING_MONTHLY_RESET_SQL);
                query.bind(list_thing.as_slice());
                query.bind(request.stamp);
                query.bind(0_i64);
                query.bind(i32::from(total_jing_li_dan_count));
                query.bind(0_i64);
                query.bind(0_i64);
                query.bind(player_id);
                query
            };
            if let Err(error) = query.execute(&mut connection).await {
                return LeiTingDatabaseResetOutcome::ReturnedFalse(
                    LeiTingDatabaseResetFailure::Database(error),
                );
            }
            updated_rows += 1;
        }

        LeiTingDatabaseResetOutcome::ReturnedTrue { updated_rows }
    }
}
