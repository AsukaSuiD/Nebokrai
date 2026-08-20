//! Владелец суточного обновления `CLeiTing` исторического WorldServer.
//!
//! Статус `getInstance` RVA `0x000A2F00`, `UpdateLeiTing` RVA `0x000A2F60`
//! и `Run` RVA `0x000A3070` — `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\leiting.cpp:25,40,76`.
//!
//! Singleton хранит единственную полную копию MSVC `tm`, полученную через
//! `_time/_localtime`. Rust получает её от caller-а и выражает singleton
//! обычным owned значением. `Run` снова снимает local `tm`, вызывает точный
//! соседний `CThingSetup::SetDailyUpdateStamp` (`23:59:59`) и сравнивает только
//! `tm_yday`; год намеренно не участвует в gate. При новом дне параметр равен
//! `(current.tm_mon != saved.tm_mon) + 1`, то есть `1` внутри месяца и `2` при
//! смене месяца.
//!
//! `UpdateLeiTing` обходит `CGame::m_mPlayer` в unsigned map-order, пропускает
//! null value и для каждого живого player строго выполняет
//! `CPlayer::UpdateLeiTing`, затем `AddByteArrayLeiTing` в message `0x7FA17`
//! после inherited player ID и синхронный `SendAll`. После всего player-
//! прохода идут begin-log, `_mktime` того же mutable `tm`, неблокирующий
//! `CRsPlayer::ResetAllLeitingInDB`, end-log и только затем замена сохранённой
//! даты. Возвраты send и DB-spawn исходный caller не читал.
//!
//! `LeiTingContext` — узкая граница ровно этих ещё сырых соседних owners. Он
//! не скрывает порядок: map keys снимаются до прохода, update и serialization
//! разделены, message строит сам `CLeiTing`, а DB callback вызывается после
//! `_mktime`. Контекст обязан синхронно скопировать message до возврата.
//! Неизвестные null/codec/time границы возвращаются как `Block`, не получая
//! придуманного fail-closed продолжения. Старые hash/STL constructors,
//! allocator, unwind и deleting-destructor blocks удалены как
//! compiler/library noise.

use std::error::Error;
use std::fmt;

use crate::nets::networld::message::CMessage;

/// Девять signed полей старого 32-bit MSVC `tm` в исходном порядке.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingLocalTime {
    pub(crate) second: i32,
    pub(crate) minute: i32,
    pub(crate) hour: i32,
    pub(crate) month_day: i32,
    pub(crate) month: i32,
    pub(crate) year_since_1900: i32,
    pub(crate) week_day: i32,
    pub(crate) year_day: i32,
    pub(crate) daylight_saving: i32,
}

/// Safe-граница неизвестного соседнего callback-а.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingBlock<ContextBlock>(pub(crate) ContextBlock);

impl<ContextBlock: fmt::Display> fmt::Display for LeiTingBlock<ContextBlock> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CLeiTing callback заблокирован: {}", self.0)
    }
}

impl<ContextBlock: Error + 'static> Error for LeiTingBlock<ContextBlock> {}

/// Выполненный новый день одного `Run`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingDailyUpdateReport {
    pub(crate) update_kind: u32,
    pub(crate) map_entries: usize,
    pub(crate) updated_players: usize,
    pub(crate) null_players: usize,
    pub(crate) database_stamp: i32,
}

/// Наблюдаемый итог одного `CLeiTing::Run`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingRunReport {
    pub(crate) current: LeiTingLocalTime,
    pub(crate) daily_update: Option<LeiTingDailyUpdateReport>,
}

/// Точные соседние вызовы полного `CLeiTing` owner-а.
pub(crate) trait LeiTingContext {
    type Block;

    /// Выполняет `CThingSetup::SetDailyUpdateStamp` над переданным `tm`.
    fn set_daily_update_stamp(&mut self, local_time: &mut LeiTingLocalTime);

    /// Выполняет отдельный `GetLocalTime` и первый formatted `AddLogText`.
    fn add_update_start_log(&mut self);

    /// Возвращает snapshot unsigned key-order `CGame::m_mPlayer`.
    fn player_map_keys(&self) -> Vec<u32>;

    /// Вызывает `CPlayer::UpdateLeiTing`; null map-value возвращает `None`.
    /// Живой player возвращает свой inherited signed ID для message.
    fn update_player(
        &mut self,
        map_key: u32,
        update_kind: u32,
        local_time: &mut LeiTingLocalTime,
    ) -> Result<Option<i32>, Self::Block>;

    /// Вызывает `CPlayer::AddByteArrayLeiTing` для того же map entry.
    fn add_player_lei_ting(
        &self,
        map_key: u32,
        destination: &mut Vec<u8>,
    ) -> Result<(), Self::Block>;

    /// Синхронно повторяет `CMessage::SendAll`; старый return игнорируется.
    fn send_all(&mut self, message: &CMessage);

    fn add_database_begin_log(&mut self);

    /// Повторяет `_mktime`, включая допустимую нормализацию mutable `tm`.
    fn mktime(&mut self, local_time: &mut LeiTingLocalTime) -> Result<i32, Self::Block>;

    /// Запускает `CRsPlayer::ResetAllLeitingInDB` без ожидания worker-а.
    fn reset_all_lei_ting_in_database(&mut self, update_kind: u32, stamp: i32);

    fn add_update_end_log(&mut self);
}

/// Owned замена process-static singleton-а и его `s_date`.
pub(crate) struct CLeiTing {
    saved_date: LeiTingLocalTime,
}

impl CLeiTing {
    /// Сохраняет полную constructor-копию одного `_localtime`.
    pub(crate) const fn new(initial_local_time: LeiTingLocalTime) -> Self {
        Self {
            saved_date: initial_local_time,
        }
    }

    /// Выполняет один полный daily gate и все достигнутые side effects.
    pub(crate) fn run<Context: LeiTingContext>(
        &mut self,
        mut current: LeiTingLocalTime,
        context: &mut Context,
    ) -> Result<LeiTingRunReport, LeiTingBlock<Context::Block>> {
        context.set_daily_update_stamp(&mut current);
        if current.year_day == self.saved_date.year_day {
            return Ok(LeiTingRunReport {
                current,
                daily_update: None,
            });
        }

        context.add_update_start_log();
        let update_kind = u32::from(current.month != self.saved_date.month) + 1;
        let map_keys = context.player_map_keys();
        let map_entries = map_keys.len();
        let mut updated_players = 0;
        let mut null_players = 0;

        for map_key in map_keys {
            let player_id = context
                .update_player(map_key, update_kind, &mut current)
                .map_err(LeiTingBlock)?;
            let Some(player_id) = player_id else {
                null_players += 1;
                continue;
            };

            let mut player_payload = Vec::new();
            context
                .add_player_lei_ting(map_key, &mut player_payload)
                .map_err(LeiTingBlock)?;
            let mut message = CMessage::new(0x0007_FA17);
            message.base_mut().add_long(player_id);
            message.base_mut().add(&player_payload);
            context.send_all(&message);
            updated_players += 1;
        }

        context.add_database_begin_log();
        let database_stamp = context.mktime(&mut current).map_err(LeiTingBlock)?;
        context.reset_all_lei_ting_in_database(update_kind, database_stamp);
        context.add_update_end_log();
        self.saved_date = current;

        Ok(LeiTingRunReport {
            current,
            daily_update: Some(LeiTingDailyUpdateReport {
                update_kind,
                map_entries,
                updated_players,
                null_players,
                database_stamp,
            }),
        })
    }
}
