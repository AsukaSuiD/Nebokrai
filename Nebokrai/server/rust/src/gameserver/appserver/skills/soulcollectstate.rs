//! Каноническое состояние сбора душ `CSoulCollectState` (`0x13B`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/soulcollectstate.cpp`. Состояние без таймера хранит
//! коэффициент и не более `skill_level` душ. Каждое успешное пополнение
//! публикует окончание прежнего снимка до нового снимка. Создание visual в Begin не отправляет
//! обновление; пакеты пары End→Begin принадлежат AddSoul. Запись в БД содержит
//! ID, уровень и число душ, но теряет `variable_percent`; после загрузки он нулевой.
//! Клиент получает нулевой remaining и число душ, не внутренний коэффициент.
//! AddSoul требует разрешимого Sufferer и использует знаковое сравнение лимита.
//!
//! End публикует visual phase2 через свежего Sufferer, пишет IsEnded=1,
//! затем заново разрешает Sufferer для RemoveState. Payload остаётся живым
//! до доставки visual. Деструктор повторяет phase2: loop1 после End остаётся
//! активным, поэтому второе снятие не подавляется. Огненные снаряды выбирают первый непустой слот ID13B,
//! сохраняют typed-параметры, вызывают End и уничтожают свежий остаток слота.
//! Несовпадение типа не отменяет End, а ended не исключает слот из поиска.
//! Object Begin вызывает базу, создаёт visual и начинает loop1 без Update:
//! повторный Begin не публикует пакет и не меняет число душ. SlotMap сохраняет
//! независимость состояний и позиции с пропусками после удаления.

use super::accumulatedstate::{
    AccumulatedState, AccumulationParticipant, add_accumulated_state, update_accumulated_visual,
};
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, end_and_destroy_state_at,
    remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;

pub(crate) const SOUL_COLLECT_STATE_ID: u32 = 0x13b;
pub(crate) const SOUL_COLLECT_STATE_BYTES: usize = 12;

pub(crate) fn add_soul_collect(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce(&CGame) -> Option<SoulCollectState>, now: &mut dyn FnMut() -> u32,
) -> bool {
    add_accumulated_state(game, source, create, now)
}

pub(crate) fn consume_soul_collect_snapshot(game: &mut CGame, source: (i32, ShapeIdentity)) -> (i32, i32) {
    let Some((position, key)) = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == SOUL_COLLECT_STATE_ID))
    else { return (0, 0); };
    let snapshot = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.applied_state::<SoulCollectState>(key))
        .map_or((0, 0), |state| (state.souls(), state.variable_percent() as i32));
    let _ = end_and_destroy_state_at(game, source.0, source.1, position);
    snapshot
}

pub(crate) fn restart_soul_collect_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<SoulCollectState>(key)).is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
        && begin_applied_state_visual(game, region_id, holder, key, 1)
}

pub(crate) fn end_soul_collect_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<SoulCollectState>(key)).is_none()
    {
        return false;
    }
    update_accumulated_visual::<SoulCollectState>(game, (region_id, holder), key, 2);
    if !resolve_state_move_shape_mut(game, region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_ended(key)) { return false; }
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key) else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, SOUL_COLLECT_STATE_BYTES)
}

pub(crate) fn destroy_soul_collect_state_visual(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) {
    update_accumulated_visual::<SoulCollectState>(game, (region_id, holder), key, 2);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SoulCollectState {
    skill_level: i32,
    variable_percent: u32,
    souls: i32,
}

impl SoulCollectState {
    pub(crate) const fn new(skill_level: i32, variable_percent: u32) -> Self {
        Self { skill_level, variable_percent, souls: 0 }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != SOUL_COLLECT_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self { skill_level: reader.read_i32()?, variable_percent: 0, souls: reader.read_i32()? })
    }

    pub(crate) fn encoded(self) -> [u8; SOUL_COLLECT_STATE_BYTES] {
        let mut bytes = [0; SOUL_COLLECT_STATE_BYTES];
        for (index, value) in [SOUL_COLLECT_STATE_ID as i32, self.skill_level, self.souls].into_iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub(crate) const fn skill_id(self) -> u32 { SOUL_COLLECT_STATE_ID }
    pub(crate) const fn variable_percent(self) -> u32 { self.variable_percent }
    pub(crate) const fn souls(self) -> i32 { self.souls }
}

impl AccumulatedState for SoulCollectState {
    const ID: u32 = SOUL_COLLECT_STATE_ID;
    const PARTICIPANT: AccumulationParticipant = AccumulationParticipant::Sufferer;

    fn increment(&mut self) -> bool {
        if self.souls >= self.skill_level { return false; }
        self.souls = self.souls.wrapping_add(1);
        true
    }

    fn record(self) -> [u8; SOUL_COLLECT_STATE_BYTES] { self.encoded() }
    fn client_fields(self) -> (u32, u32) { (0, self.souls as u32) }
}
