//! Владелец суточного обновления `CLeiTing` исторического WorldServer.
//!
//! и `Run` — часть контракта owner-а. Источник контракта — точная пара WorldServer EXE/PDB.
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
//! `CGame`, `CPlayer`, `CThingSetup` и globe snapshot теперь являются
//! concrete owners этого прохода. `LeiTingContext` оставляет только platform-
//! time, log, transport и DB worker границы: map keys снимает `CGame`, player
//! update/serialization выполняет `CPlayer`, message строит `CLeiTing`.
//! Контекст обязан синхронно скопировать message до возврата. Неизвестные
//! codec/time границы возвращаются как typed `Block`; старые hash/STL
//! constructors, allocator, unwind и deleting-destructor blocks удалены как
//! compiler/library noise.

use std::error::Error;
use std::fmt;

use crate::nets::networld::message::CMessage;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::leitingsetup::CThingSetup;
pub(crate) use crate::setup::leitingsetup::LeiTingLocalTime;
use crate::worldserver::appworld::player::{
    PlayerCodecError, PlayerLeiTingClock, PlayerLeiTingUpdateBlock,
};
use crate::worldserver::worldserver::game::CGame;

/// Safe-граница неизвестного соседнего callback-а.
#[derive(Debug)]
pub(crate) enum LeiTingBlock<ContextBlock> {
    Context(ContextBlock),
    Player(PlayerLeiTingUpdateBlock<ContextBlock>),
    PlayerCodec(PlayerCodecError),
}

impl<ContextBlock: fmt::Display> fmt::Display for LeiTingBlock<ContextBlock> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Context(source) => write!(formatter, "CLeiTing callback заблокирован: {source}"),
            Self::Player(PlayerLeiTingUpdateBlock::PreviousLocalTime(source)) => write!(
                formatter,
                "CPlayer::UpdateLeiTing не получил previous localtime: {source}"
            ),
            Self::Player(PlayerLeiTingUpdateBlock::Stamp(source)) => write!(
                formatter,
                "CPlayer::UpdateLeiTing не нормализовал новый stamp: {source}"
            ),
            Self::PlayerCodec(source) => {
                write!(formatter, "CPlayer::AddByteArrayLeiTing заблокирован: {source}")
            }
        }
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

 /// Выполняет отдельный `GetLocalTime` и первый formatted `AddLogText`.
    fn add_update_start_log(&mut self);

 /// Platform replacement 32-bit CRT `_localtime` для player stamp.
    fn local_time_from_timestamp(
        &mut self,
        timestamp: u32,
    ) -> Result<LeiTingLocalTime, Self::Block>;

 /// Отдельный Win32 `GetLocalTime().wDayOfWeek` внутри weekly item loop.
    fn current_week_day(&mut self) -> u16;

 /// Синхронно повторяет `CMessage::SendAll`; старый return игнорируется.
    fn send_all(&mut self, message: &CMessage);

    fn add_database_begin_log(&mut self);

 /// Повторяет `_mktime`, включая допустимую нормализацию mutable `tm`.
    fn mktime(&mut self, local_time: &mut LeiTingLocalTime) -> Result<i32, Self::Block>;

 /// Запускает `CRsPlayer::ResetAllLeitingInDB` без ожидания worker-а.
    fn reset_all_lei_ting_in_database(&mut self, update_kind: u32, stamp: i32);

    fn add_update_end_log(&mut self);
}

impl<Context: LeiTingContext + ?Sized> PlayerLeiTingClock for Context {
    type Block = Context::Block;

    fn local_time_from_timestamp(
        &mut self,
        timestamp: u32,
    ) -> Result<LeiTingLocalTime, Self::Block> {
        LeiTingContext::local_time_from_timestamp(self, timestamp)
    }

    fn mktime(&mut self, local_time: &mut LeiTingLocalTime) -> Result<i32, Self::Block> {
        LeiTingContext::mktime(self, local_time)
    }

    fn current_week_day(&mut self) -> u16 {
        LeiTingContext::current_week_day(self)
    }
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

 /// Выполняет один полный daily gate и все действующие side effects.
    pub(crate) fn run<Context: LeiTingContext>(
        &mut self,
        mut current: LeiTingLocalTime,
        game: &mut CGame,
        globe_setup: &GlobeSetupSnapshot,
        context: &mut Context,
    ) -> Result<LeiTingRunReport, LeiTingBlock<Context::Block>> {
        CThingSetup::set_daily_update_stamp(&mut current);
        if current.year_day == self.saved_date.year_day {
            return Ok(LeiTingRunReport {
                current,
                daily_update: None,
            });
        }

        context.add_update_start_log();
        let update_kind = u32::from(current.month != self.saved_date.month) + 1;
        let map_keys = game.player_map_keys();
        let map_entries = map_keys.len();
        let mut updated_players = 0;
        let mut null_players = 0;

        for map_key in map_keys {
            let update = game
                .update_map_player_lei_ting(
                    map_key,
                    update_kind,
                    &mut current,
                    globe_setup,
                    context,
                )
                .map_err(LeiTingBlock::Player)?;
            let Some(update) = update else {
                null_players += 1;
                continue;
            };

            let mut player_payload = Vec::new();
            game.map_player(map_key)
                .expect("неизменный map-owner только что выполнил UpdateLeiTing")
                .add_byte_array_lei_ting(&mut player_payload)
                .map_err(LeiTingBlock::PlayerCodec)?;
            let mut message = CMessage::new(0x0007_FA17);
            message.base_mut().add_long(update.player_id);
            message.base_mut().add(&player_payload);
            context.send_all(&message);
            updated_players += 1;
        }

        context.add_database_begin_log();
        let database_stamp = LeiTingContext::mktime(context, &mut current)
            .map_err(LeiTingBlock::Context)?;
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
