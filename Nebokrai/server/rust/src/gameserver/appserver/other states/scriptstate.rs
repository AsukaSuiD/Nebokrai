//! Семь состояний CMoveShape::AddState, gameserver.exe + GameServer.pdb.
//! Исходные owners: moveshape.cpp и other states/{usegoodsenlarge*,improveexp,
//! autoprotect}state.cpp. Здесь один lifecycle/codec и одна запись арены:
//! started/keep хранятся только у ScriptMoveState, коэффициент — в варианте.
//! Чистые формулы остаются в исходных owners; экземпляры не копируются.
//! Пять UseGoods и ImproveExp требуют non-NULL User до base Begin. Первичное
//! self/self применение читает clock до U/S getters, создаёт visual loop0,
//! затем caller добавляет запись и вызывает UpdateProperty. Begin не шлёт
//! пакет; первый property visual шлёт BFE03 и завершает one-shot ресурс.
//! Begin(NULL, holder) этих шести возвращает false без мутаций и часов.
//! AutoProtect Begin0x005D4290 требует non-NULL S и отклоняет Player-GM:
//! base → visual loop1, без флага/пакета. Его NULL-user restart не читает часы.
//! Общая арена хранит остаточный visual без повторного Begin/публикации.
//! AI0x005D5BA0/EXP0x005D60B0: один clock → unsigned start+keep<now → End,
//! без ended/death/owner-gates и без особого случая keep0. End шести
//! 0x005D5B80: ended у записи → actual S → RemoveState(pointer), без visual.
//! AutoProtect End0x005D44E0 сохраняет первого Player-S, проверяет non-GM,
//! вызывает optional visual1 со свежим S, сбрасывает флаг первому S и удаляет
//! через него; state.ended не меняется. Чужая арена не получает local StateKey.
//! Property пяти UseGoods сохраняет первого S: NULL→false, optional visual0,
//! затем тип/формула на первом S с живым коэффициентом после visual. ImproveExp
//! property0x005D5F10 вызывает только visual0, формула опыта читается отдельно.
//! AutoProtect property0x005D4240: первый Player-S/non-GM → flag=true → visual0.
//! Все visuals читают actual S; ended/NULL подавляют пакет, но не base tail.
//! Шесть visuals всегда строят BFE03; AutoProtect visual0x005D4530 различает
//! BFE03/BFE04. Время — два чтения при положительном остатке, additional=0.
//! SetRegion пяти UseGoods и AutoProtect меняет User; ImproveExp — Sufferer.
//! Save шести0x005D4D10/0x005E7330 пишет ID/remaining/coefficient (12 байт),
//! AutoProtect0x005F51E0 — ID/remaining (8 байт). Save не меняет live keep.
//! Decode0x004F9D80/0x005D6190/0x005EAAC0 читает clock перед keep/coefficient.
//! SlotMap/заимствования заменяют CState* и ручной lifetime; отказ native
//! allocator не эмулируется. Неизвестные перегрузки Begin сохранены у owners.

use super::autoprotectstate::AUTO_PROTECT_STATE_ID;
use super::improveexpstate::{self, IMPROVE_EXP_STATE_ID};
use super::player::PlayerCombatProperties;
use super::shape::ShapeIdentity;
use super::usegoodsenlargedefstate::{self, USE_GOODS_ENLARGE_DEF_STATE_ID};
use super::usegoodsenlargeelmdefstate::{self, USE_GOODS_ENLARGE_ELM_DEF_STATE_ID};
use super::usegoodsenlargefullmissstate::{self, USE_GOODS_ENLARGE_FULL_MISS_STATE_ID};
use super::usegoodsenlargemaxhpstate::{self, USE_GOODS_ENLARGE_MAX_HP_STATE_ID};
use super::usegoodsenlargemaxmpstate::{self, USE_GOODS_ENLARGE_MAX_MP_STATE_ID};
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time,
    update_applied_state_end_visual, update_property_state_visual,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::public::guid::CGuid;

pub(crate) const SCRIPT_STATE_TIMED_BYTES: usize = 12;
pub(crate) const AUTO_PROTECT_STATE_BYTES: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
enum ScriptStateKind {
    EnlargeMaxHp(u32),
    EnlargeMaxMp(u32),
    ImproveExp(u32),
    EnlargeDefense(u32),
    EnlargeElementDefense(u32),
    EnlargeFullMiss(u32),
    AutoProtect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScriptMoveState {
    kind: ScriptStateKind,
    started_at_ms: u32,
    time_to_keep_ms: u32,
}

impl ScriptMoveState {
    pub(crate) fn from_factory(state_id: i32, value1: i32, value2: i32) -> Option<Self> {
        let coefficient = value2 as u32;
        let kind = match state_id {
            USE_GOODS_ENLARGE_MAX_HP_STATE_ID => ScriptStateKind::EnlargeMaxHp(coefficient),
            USE_GOODS_ENLARGE_MAX_MP_STATE_ID => ScriptStateKind::EnlargeMaxMp(coefficient),
            IMPROVE_EXP_STATE_ID => ScriptStateKind::ImproveExp(coefficient),
            USE_GOODS_ENLARGE_DEF_STATE_ID => ScriptStateKind::EnlargeDefense(coefficient),
            USE_GOODS_ENLARGE_ELM_DEF_STATE_ID => ScriptStateKind::EnlargeElementDefense(coefficient),
            USE_GOODS_ENLARGE_FULL_MISS_STATE_ID => ScriptStateKind::EnlargeFullMiss(coefficient),
            AUTO_PROTECT_STATE_ID => ScriptStateKind::AutoProtect,
            _ => return None,
        };
        Some(Self { kind, started_at_ms: 0, time_to_keep_ms: value1 as u32 })
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let state_id = reader.read_i32()?;
        let keep_time = reader.read_u32()?;
        let value = if state_id == AUTO_PROTECT_STATE_ID { 0 } else { reader.read_u32()? };
        let mut state = Self::from_factory(state_id, keep_time as i32, value as i32)
            .ok_or(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            })?;
        state.started_at_ms = now_ms;
        Ok(state)
    }

    pub(crate) const fn serialized_size(state_id: i32) -> Option<usize> {
        match state_id {
            AUTO_PROTECT_STATE_ID => Some(AUTO_PROTECT_STATE_BYTES),
            USE_GOODS_ENLARGE_MAX_HP_STATE_ID
            | USE_GOODS_ENLARGE_MAX_MP_STATE_ID
            | IMPROVE_EXP_STATE_ID
            | USE_GOODS_ENLARGE_DEF_STATE_ID
            | USE_GOODS_ENLARGE_ELM_DEF_STATE_ID
            | USE_GOODS_ENLARGE_FULL_MISS_STATE_ID => Some(SCRIPT_STATE_TIMED_BYTES),
            _ => None,
        }
    }

    fn coefficient(&self) -> Option<u32> {
        match &self.kind {
            ScriptStateKind::EnlargeMaxHp(value)
            | ScriptStateKind::EnlargeMaxMp(value)
            | ScriptStateKind::ImproveExp(value)
            | ScriptStateKind::EnlargeDefense(value)
            | ScriptStateKind::EnlargeElementDefense(value)
            | ScriptStateKind::EnlargeFullMiss(value) => Some(*value),
            ScriptStateKind::AutoProtect => None,
        }
    }

    fn encoded_with_remaining(&self, remaining: u32) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(if self.is_auto_protect() {
            AUTO_PROTECT_STATE_BYTES
        } else {
            SCRIPT_STATE_TIMED_BYTES
        });
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_i32(self.state_id());
        writer.write_u32(remaining);
        if let Some(value) = self.coefficient() { writer.write_u32(value); }
        bytes
    }

    pub(crate) fn encoded(&self, now: impl FnMut() -> u32) -> Vec<u8> {
        self.encoded_with_remaining(self.client_state_time(now) as u32)
    }

    pub(crate) fn encoded_for_install(&self) -> Vec<u8> {
        self.encoded_with_remaining(self.time_to_keep_ms)
    }

    pub(crate) const fn state_id(&self) -> i32 {
        match &self.kind {
            ScriptStateKind::EnlargeMaxHp(_) => USE_GOODS_ENLARGE_MAX_HP_STATE_ID,
            ScriptStateKind::EnlargeMaxMp(_) => USE_GOODS_ENLARGE_MAX_MP_STATE_ID,
            ScriptStateKind::ImproveExp(_) => IMPROVE_EXP_STATE_ID,
            ScriptStateKind::EnlargeDefense(_) => USE_GOODS_ENLARGE_DEF_STATE_ID,
            ScriptStateKind::EnlargeElementDefense(_) => USE_GOODS_ENLARGE_ELM_DEF_STATE_ID,
            ScriptStateKind::EnlargeFullMiss(_) => USE_GOODS_ENLARGE_FULL_MISS_STATE_ID,
            ScriptStateKind::AutoProtect => AUTO_PROTECT_STATE_ID,
        }
    }

    pub(crate) const fn is_auto_protect(&self) -> bool {
        matches!(self.kind, ScriptStateKind::AutoProtect)
    }

    pub(crate) const fn is_improve_exp(&self) -> bool {
        matches!(self.kind, ScriptStateKind::ImproveExp(_))
    }

    pub(crate) fn client_state_time(&self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.time_to_keep_ms, now) as i32
    }

    pub(crate) const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.time_to_keep_ms) < now_ms
    }

    fn apply_combat_properties(&self, properties: &mut PlayerCombatProperties) {
        match &self.kind {
            ScriptStateKind::EnlargeMaxHp(value) => usegoodsenlargemaxhpstate::apply(*value, properties),
            ScriptStateKind::EnlargeMaxMp(value) => usegoodsenlargemaxmpstate::apply(*value, properties),
            ScriptStateKind::EnlargeDefense(value) => usegoodsenlargedefstate::apply(*value, properties),
            ScriptStateKind::EnlargeElementDefense(value) => usegoodsenlargeelmdefstate::apply(*value, properties),
            ScriptStateKind::EnlargeFullMiss(value) => usegoodsenlargefullmissstate::apply(*value, properties),
            ScriptStateKind::ImproveExp(_) | ScriptStateKind::AutoProtect => {}
        }
    }

    pub(crate) fn experience_multiplier_delta(&self) -> f64 {
        match &self.kind {
            ScriptStateKind::ImproveExp(value) => improveexpstate::multiplier_delta(*value),
            _ => 0.0,
        }
    }
}

pub(crate) fn begin_primary_script_state(
    game: &mut CGame,
    player_id: i32,
    state_id: i32,
    value1: i32,
    value2: i32,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let mut state = ScriptMoveState::from_factory(state_id, value1, value2)?;
    game.find_player(player_id)?;
    if state.is_auto_protect() && game.script_player_gm_level(player_id).unwrap_or(0) != 0 {
        return None;
    }
    state.started_at_ms = now();
    let player = game.find_player(player_id)?;
    let participant = (
        player.shape().get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..player.shape().identity() },
    );
    let record = state.encoded_for_install();
    let shape = game.find_player_mut(player_id)?.move_shape_mut();
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(participant));
    shape.set_applied_state_sufferer(key, Some(participant));
    Some(key)
}

pub(crate) fn update_script_move_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
    else { return false; };
    if state.is_auto_protect() {
        let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
        else { return true; };
        if target.object_type != 400 || game.script_player_gm_level(target.id).unwrap_or(0) != 0 {
            return true;
        }
        let Some(player) = game.find_player_mut(target.id) else { return true; };
        player.set_auto_protected(true);
    } else if !state.is_improve_exp() {
        let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
        else { return false; };
        update_script_state_begin_visual(game, region_id, holder, key, now);
        if target.object_type == 400 {
            let Some(player) = game.find_player(target.id) else { return false; };
            let mut properties = player.combat_properties();
            let Some(state) = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
            else { return false; };
            state.apply_combat_properties(&mut properties);
            let Some(player) = game.find_player_mut(target.id) else { return false; };
            player.update_state_combat_properties(|_| properties);
        }
        return true;
    }
    update_script_state_begin_visual(game, region_id, holder, key, now);
    true
}

fn update_script_state_begin_visual(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    now: &mut dyn FnMut() -> u32,
) {
    update_property_state_visual::<ScriptMoveState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_state_time(now) as u32,
    );
}

pub(crate) fn restart_script_move_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
    else { return false; };
    if !state.is_auto_protect()
        || (holder.object_type == 400 && game.script_player_gm_level(holder.id).unwrap_or(0) != 0)
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_script_move_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    let sampled_at_ms = runtime.now_milliseconds();
    let expired = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
        .is_some_and(|state| state.expired(sampled_at_ms));
    expired && end_script_move_state(game, region_id, holder, key)
}

pub(crate) fn end_script_move_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
    else { return false; };
    if state.is_auto_protect() {
        let Some(target @ (_, identity)) = resolve_applied_state_sufferer(game, region_id, holder, key)
        else { return false; };
        if identity.object_type != 400 || game.script_player_gm_level(identity.id).unwrap_or(0) != 0 {
            return false;
        }
        update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
        let Some(player) = game.find_player_mut(identity.id) else { return false; };
        player.set_auto_protected(false);
        remove_applied_state_from(game, region_id, holder, key, target, AUTO_PROTECT_STATE_BYTES)
    } else {
        if !resolve_state_move_shape_mut(game, region_id, holder)
            .is_some_and(|shape| shape.mark_applied_state_ended(key))
        {
            return false;
        }
        let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
        else { return false; };
        remove_applied_state_from(game, region_id, holder, key, target, SCRIPT_STATE_TIMED_BYTES)
    }
}
