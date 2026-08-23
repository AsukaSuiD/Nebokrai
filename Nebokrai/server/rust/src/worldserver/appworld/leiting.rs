//! Суточное обновление `CLeiTing` из WorldServer, подтверждённое
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! `Run` сравнивает только `tm_yday`; при новом дне parameter равен 1 внутри
//! месяца и 2 при смене месяца. Players обходятся в unsigned map order:
//! сначала `UpdateLeiTing`, затем serialization в `0x7FA17` и `SendAll`.
//!
//! После всех players идут begin-log, `_mktime`, неблокирующий DB reset,
//! end-log и только затем замена сохранённой даты. Send/DB-spawn results
//! оригинал игнорировал. Platform time и transport передаются через контекст;
//! само игровое состояние остаётся у `CGame`, `CPlayer` и `CThingSetup`.

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingDailyUpdateReport {
    pub(crate) update_kind: u32,
    pub(crate) map_entries: usize,
    pub(crate) updated_players: usize,
    pub(crate) null_players: usize,
    pub(crate) database_stamp: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingRunReport {
    pub(crate) current: LeiTingLocalTime,
    pub(crate) daily_update: Option<LeiTingDailyUpdateReport>,
}

pub(crate) trait LeiTingContext {
    type Block;

    fn add_update_start_log(&mut self);

    fn local_time_from_timestamp(
        &mut self,
        timestamp: u32,
    ) -> Result<LeiTingLocalTime, Self::Block>;

    fn current_week_day(&mut self) -> u16;

    fn send_all(&mut self, message: &CMessage);

    fn add_database_begin_log(&mut self);

    fn mktime(&mut self, local_time: &mut LeiTingLocalTime) -> Result<i32, Self::Block>;

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

pub(crate) struct CLeiTing {
    saved_date: LeiTingLocalTime,
}

impl CLeiTing {
    pub(crate) const fn new(initial_local_time: LeiTingLocalTime) -> Self {
        Self {
            saved_date: initial_local_time,
        }
    }

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
