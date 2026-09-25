//! Суточное обновление `CLeiTing` из WorldServer, перенесённое в Realm
//! `activities/`, подтверждённое `worldserver.exe` и `worldserver.pdb`.
//!
//! `Run` сравнивает только `tm_yday`; при новом дне parameter равен 1 внутри
//! месяца и 2 при смене месяца. Players обходятся в unsigned map order:
//! сначала `UpdateLeiTing`, затем serialization в `0x7FA17` и `SendAll`.
//!
//! После всех players идут begin-log, `_mktime`, неблокирующий DB reset,
//! end-log и только затем замена сохранённой даты. Send/DB-spawn results
//! оригинал игнорировал. Platform time и transport передаются через контекст;
//! само игровое состояние остаётся у `CGame`, `CPlayer` и `CThingSetup`.
//!
//! Прямые ссылки на `CGame`/`CPlayer` заменены двумя узкими typed-швами
//! `LeiTingGameView` и `LeiTingPlayerCodec`; реализации живут у старых
//! владельцев и делегируют их inherent-методам.

use std::error::Error;
use std::fmt;

use crate::app::world_message::CMessage;
use nebokrai_shared::resources::{CThingSetup, GlobeSetupSnapshot};
pub use nebokrai_shared::resources::LeiTingLocalTime;

pub trait PlayerLeiTingClock {
    type Block;

    fn local_time_from_timestamp(
        &mut self,
        timestamp: u32,
    ) -> Result<LeiTingLocalTime, Self::Block>;

    fn mktime(&mut self, local_time: &mut LeiTingLocalTime) -> Result<i32, Self::Block>;

    fn current_week_day(&mut self) -> u16;
}

#[derive(Debug)]
pub enum PlayerLeiTingUpdateBlock<ClockBlock> {
    PreviousLocalTime(ClockBlock),
    Stamp(ClockBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerLeiTingUpdateReport {
    pub player_id: i32,
    pub update_kind: u32,
    pub previous_stamp: u32,
    pub resulting_stamp: u32,
    pub stamp_replaced: bool,
    pub daily_list_replaced: bool,
    pub daily_thing_count: usize,
}

/// Узкая codec-точка игрока для fan-out daily update. Реализация живёт у
/// владельца игрока (старый `CPlayer`), имя совпадает с inherent-методом
/// специально (path-call на старой стороне резолвит inherent по приоритету,
/// рекурсии нет). Конкретная ошибка codec-стены владельца — `PlayerCodecError`
/// из ещё не перенесённой цепочки containers/goods/shape старого `player.rs` —
/// проходит швом как associated `Block`.
pub trait LeiTingPlayerCodec {
    type Block: Error + 'static;

    fn add_byte_array_lei_ting(&self, destination: &mut Vec<u8>) -> Result<(), Self::Block>;
}

/// Узкий game-view daily reset LeiTing. Реализация живёт у владельца игры
/// (старый `CGame`) и делегирует inherent-методам.
pub trait LeiTingGameView {
    type Player: LeiTingPlayerCodec;

    fn lei_ting_player_map_keys(&self) -> Vec<u32>;

    fn lei_ting_update_map_player<Clock: PlayerLeiTingClock>(
        &mut self,
        map_key: u32,
        update_kind: u32,
        stamp: &mut LeiTingLocalTime,
        globe_setup: &GlobeSetupSnapshot,
        clock: &mut Clock,
    ) -> Result<Option<PlayerLeiTingUpdateReport>, PlayerLeiTingUpdateBlock<Clock::Block>>;

    fn lei_ting_map_player(&self, map_key: u32) -> Option<&Self::Player>;
}

#[derive(Debug)]
pub enum LeiTingBlock<ContextBlock, CodecBlock> {
    Context(ContextBlock),
    Player(PlayerLeiTingUpdateBlock<ContextBlock>),
    PlayerCodec(CodecBlock),
}

impl<ContextBlock: fmt::Display, CodecBlock: fmt::Display> fmt::Display
    for LeiTingBlock<ContextBlock, CodecBlock>
{
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

impl<ContextBlock: Error + 'static, CodecBlock: Error + 'static> Error
    for LeiTingBlock<ContextBlock, CodecBlock>
{
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeiTingDailyUpdateReport {
    pub update_kind: u32,
    pub map_entries: usize,
    pub updated_players: usize,
    pub null_players: usize,
    pub database_stamp: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeiTingRunReport {
    pub current: LeiTingLocalTime,
    pub daily_update: Option<LeiTingDailyUpdateReport>,
}

pub trait LeiTingContext {
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

pub struct CLeiTing {
    saved_date: LeiTingLocalTime,
}

impl CLeiTing {
    pub const fn new(initial_local_time: LeiTingLocalTime) -> Self {
        Self {
            saved_date: initial_local_time,
        }
    }

    pub fn run<Context: LeiTingContext, Game: LeiTingGameView + ?Sized>(
        &mut self,
        mut current: LeiTingLocalTime,
        game: &mut Game,
        globe_setup: &GlobeSetupSnapshot,
        context: &mut Context,
    ) -> Result<
        LeiTingRunReport,
        LeiTingBlock<Context::Block, <Game::Player as LeiTingPlayerCodec>::Block>,
    > {
        CThingSetup::set_daily_update_stamp(&mut current);
        if current.year_day == self.saved_date.year_day {
            return Ok(LeiTingRunReport {
                current,
                daily_update: None,
            });
        }

        context.add_update_start_log();
        let update_kind = u32::from(current.month != self.saved_date.month) + 1;
        let map_keys = game.lei_ting_player_map_keys();
        let map_entries = map_keys.len();
        let mut updated_players = 0;
        let mut null_players = 0;

        for map_key in map_keys {
            let update = game
                .lei_ting_update_map_player(
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
            game.lei_ting_map_player(map_key)
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
